mod cli;
mod error;
mod state;
mod template;

use axum::{Router, routing::get};

use bytes::BytesMut;
use clap::Parser;
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
use crate::{
    cli::Args,
    state::{InnerState, event_handler, init, root},
};

#[cfg(target_env = "musl")]
use mimalloc::MiMalloc;

#[cfg(target_env = "musl")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

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

fn check_uniqueness(port: u16) -> eyre::Result<()> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("glypho");
    let pid_file = xdg_dirs.find_runtime_file("running.pid");

    match pid_file {
        //do client mode
        Some(_) => exit(0),
        //create file and start server
        None => {
            let pid = xdg_dirs
                .place_runtime_file("running.pid")
                .expect("cannot create configuration directory");
            let mut runtime_file = File::create(pid)?;
            write!(&mut runtime_file, "port={port}")?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    logger();
    let args = Args::parse();

    let port = args.port.unwrap_or_else(|| 0);

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

    check_uniqueness(port)?;
    info!("Starting Glypho...");

    let shared_state = Arc::new(Mutex::new(InnerState::new(
        file.clone(),
        BytesMut::with_capacity(4096),
    )?));

    let serve_dir = ServeDir::new(file.parent().unwrap());
    let router = Router::new()
        .route("/", get(root))
        .route("/init", get(init))
        .fallback_service(serve_dir)
        .route("/sse", get(event_handler))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    let local_addr = listener.local_addr()?;

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

    println!("");
}
