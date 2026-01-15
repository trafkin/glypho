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

#[cfg(target_env = "musl")]
use mimalloc::MiMalloc;

#[cfg(target_env = "musl")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Serialize, Deserialize)]
struct ProcessStatus {
    pub port: u16,
    pub pid: u32,
}

fn cleanup() -> eyre::Result<()> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("glypho");
    let pid_file = xdg_dirs.find_runtime_file("running.pid");
    if let Some(file) = pid_file {
        std::fs::remove_file(file)?;
    } else {
        info!("Pid file cannot be removed");
    }

    Ok(())
}

async fn check_uniqueness(file_to_add: PathBuf) -> eyre::Result<()> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("glypho");
    let pid_file = xdg_dirs.find_runtime_file("running.pid");

    match pid_file {
        //do client mode
        Some(pid_file) => {
            let file = tokio::fs::read_to_string(pid_file).await?;
            let ps: ProcessStatus = toml::from_str(file.as_str())?;
            let pid = ps.pid;
            let process_dir = format!("/proc/{pid}");

            if std::fs::exists(PathBuf::from(process_dir))? {
                let port = ps.port;
                let client = reqwest::Client::new();
                let _res = client
                    .post(format!("http://localhost:{port}/add"))
                    .json(&AddFileRequest {
                        file: file_to_add.clone(),
                    })
                    .send()
                    .await?;
                exit(0)
            }
        }
        //create file and start server
        None => (),
    }

    Ok(())
}

fn write_runtime(port: u16) -> eyre::Result<()> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("glypho");

    let pid_file = xdg_dirs
        .place_runtime_file("running.pid")
        .expect("cannot create configuration directory");

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

    let _ = check_uniqueness(file.clone()).await?;
    info!("Starting Glypho...");

    let shared_state = Arc::new(Mutex::new(InnerState::new(file.clone())));

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
