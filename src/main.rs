mod cli;
mod error;
mod state;
mod template;
mod wikilinks;

use axum::routing::post;
use axum::{Router, routing::get};

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::process::exit;
use std::sync::Arc;
use std::{env, path::PathBuf};
use tokio::sync::Mutex;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::error::GlyphoError;
use crate::state::{AddFileRequest, add_file, change_active};
use crate::{
    cli::Args,
    state::{InnerState, event_handler, root},
};

// #[cfg(target_env = "musl")]
// use mimalloc::MiMalloc;
//
// #[cfg(target_env = "musl")]
// #[global_allocator]
// static GLOBAL: MiMalloc = MiMalloc;

#[derive(Serialize, Deserialize)]
struct ProcessStatus {
    pub port: u16,
    pub pid: u32,
}

/// Directory holding runtime files (pidfile). Prefers the platform runtime
/// dir (Linux: $XDG_RUNTIME_DIR/glypho), falling back to the state/data dir
/// where no runtime-dir concept exists (macOS, Windows).
fn runtime_dir() -> eyre::Result<PathBuf> {
    use etcetera::{AppStrategy, AppStrategyArgs, choose_app_strategy};

    let strategy = choose_app_strategy(AppStrategyArgs {
        top_level_domain: "dev".to_string(),
        author: "trafkin".to_string(),
        app_name: "glypho".to_string(),
    })?;

    Ok(strategy
        .runtime_dir()
        .or_else(|| strategy.state_dir())
        .unwrap_or_else(|| strategy.data_dir()))
}

fn pidfile_path() -> eyre::Result<PathBuf> {
    Ok(runtime_dir()?.join("running.pid"))
}

fn cleanup() -> eyre::Result<()> {
    let pid_file = pidfile_path()?;
    if pid_file.exists() {
        std::fs::remove_file(pid_file)?;
    } else {
        info!("Pid file cannot be removed");
    }

    Ok(())
}

async fn check_uniqueness(file_to_add: PathBuf) -> eyre::Result<()> {
    let pidfile = pidfile_path()?;
    if !pidfile.exists() {
        return Ok(());
    }

    let file = tokio::fs::read_to_string(&pidfile).await?;
    let ps: ProcessStatus = toml::from_str(file.as_str())?;
    let port = ps.port;

    // Liveness probe: the thing we care about is whether a glypho server is
    // answering on the recorded port, not whether some pid exists (pids are
    // reused; /proc is Linux-only). A short timeout keeps startup snappy when
    // the pidfile is stale.
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()?;

    let alive = client
        .get(format!("http://localhost:{port}/"))
        .send()
        .await
        .is_ok();

    if alive {
        // client mode: hand the file to the running instance
        let _res = client
            .post(format!("http://localhost:{port}/add"))
            .json(&AddFileRequest {
                file: file_to_add.clone(),
            })
            .send()
            .await?;
        exit(0)
    }

    // Stale pidfile: remove it and continue as the primary instance.
    std::fs::remove_file(&pidfile)?;
    Ok(())
}

fn write_runtime(port: u16) -> eyre::Result<()> {
    let pid_file = pidfile_path()?;
    if let Some(parent) = pid_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut runtime_file = File::create(pid_file)?;
    let pid = std::process::id();

    let ps = ProcessStatus { port, pid };
    let toml_string = toml::to_string(&ps)?;

    write!(&mut runtime_file, "{toml_string}")?;
    Ok(())
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    logger();
    let args = Args::parse();

    let port = args.port.unwrap_or(0);

    let theme_css = args
        .theme
        .and_then(|path| std::fs::read_to_string(&path).ok());

    let file = match args.input {
        Some(f) => {
            if f.is_file() {
                PathBuf::from(f.filename())
            } else {
                return Err(GlyphoError::NotProvided.into());
            }
        }

        None => return Err(GlyphoError::NotProvided.into()),
    };

    check_uniqueness(file.clone()).await?;
    info!("Starting Glypho...");

    let mut inner_state = InnerState::new(file.clone());
    inner_state.set_theme_css(theme_css);
    let shared_state = Arc::new(Mutex::new(inner_state));

    let serve_dir = ServeDir::new(file.parent().unwrap());
    let router = Router::new()
        .route("/", get(root))
        // .route("/init", get(init))
        .fallback_service(serve_dir)
        .route("/sse", get(event_handler))
        .route("/add", post(add_file))
        .route("/update", get(change_active))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    let local_addr = listener.local_addr()?;
    write_runtime(local_addr.port())?;

    let file_name = file
        .file_name()
        .and_then(|fname| fname.to_str())
        .unwrap_or("unknown");
    tracing::info!(
        "Serving {} at http://{}",
        file_name,
        listener.local_addr().unwrap()
    );

    info!("Press Ctrl+C to stop the server");

    if !args.no_browser {
        open::that_detached(format!("http://{local_addr}"))?;
    }

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            cleanup()?;
            info!("Shutting down the server");
        }
        _ = axum::serve(listener, router) => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex as StdMutex;

    /// Env-mutating tests must not run in parallel with each other.
    static ENV_LOCK: StdMutex<()> = StdMutex::new(());

    struct EnvGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
        _tmp: tempfile::TempDir,
        saved: Vec<(&'static str, Option<std::ffi::OsString>)>,
    }

    impl EnvGuard {
        /// Point XDG/HOME at a temp dir so `runtime_dir()` resolves inside it.
        fn new() -> Self {
            let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            let tmp = tempfile::TempDir::new().unwrap();
            let keys: [&'static str; 3] = ["XDG_RUNTIME_DIR", "XDG_STATE_HOME", "HOME"];
            let saved = keys.iter().map(|k| (*k, env::var_os(k))).collect();

            unsafe {
                env::set_var("XDG_RUNTIME_DIR", tmp.path().join("run"));
                env::set_var("XDG_STATE_HOME", tmp.path().join("state"));
                env::set_var("HOME", tmp.path());
            }
            std::fs::create_dir_all(tmp.path().join("run")).unwrap();

            Self {
                _lock: lock,
                _tmp: tmp,
                saved,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (k, v) in &self.saved {
                unsafe {
                    match v {
                        Some(val) => env::set_var(k, val),
                        None => env::remove_var(k),
                    }
                }
            }
        }
    }

    #[test]
    fn pidfile_round_trip() {
        let _guard = EnvGuard::new();

        write_runtime(4321).unwrap();

        let pidfile = pidfile_path().unwrap();
        assert!(pidfile.exists(), "pidfile should be written");
        let content = std::fs::read_to_string(&pidfile).unwrap();
        let ps: ProcessStatus = toml::from_str(&content).unwrap();
        assert_eq!(ps.port, 4321);
        assert_eq!(ps.pid, std::process::id());

        // cleanup removes it
        cleanup().unwrap();
        assert!(!pidfile.exists(), "cleanup should remove the pidfile");
    }

    #[test]
    fn runtime_dir_prefers_xdg_runtime_dir() {
        let guard = EnvGuard::new();
        let dir = runtime_dir().unwrap();
        // ProjectDirs joins the application name under XDG_RUNTIME_DIR
        assert_eq!(dir, guard._tmp.path().join("run").join("glypho"));
    }

    #[tokio::test]
    async fn stale_pidfile_is_removed_and_startup_continues() {
        let _guard = EnvGuard::new();

        // Write a pidfile pointing at a port nothing listens on.
        write_runtime(1).unwrap();
        let pidfile = pidfile_path().unwrap();
        assert!(pidfile.exists());

        let file = PathBuf::from("some.md");
        check_uniqueness(file).await.unwrap();

        assert!(!pidfile.exists(), "stale pidfile should be removed");
    }
}

fn logger() {
    // If you want to see debug logs define the env var as GLYPHO=debug
    let log_level = env::var("GLYPHO").unwrap_or_else(|_| "info".into());

    let is_debug = log_level == "debug";

    // Logger
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .without_time()
                .with_file(is_debug)
                .with_line_number(is_debug)
                .with_target(is_debug)
                .with_level(is_debug),
        )
        .with(
            EnvFilter::try_new(format!("glypho={}", log_level))
                .expect("error in EnvFilter (Logger)"),
        )
        .init();
}
