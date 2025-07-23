mod template;
mod state;
mod cli;

use axum::{
    Router,
    routing::get,
};

use bytes::BytesMut;
use clap::Parser;
use tracing::info;
use std::{env, 
    sync::{Arc, Mutex}};
use tower_http::services::ServeDir;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::{cli::{Args, Cmds}, state::{event_handler, root, InnerState}};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    logger();
    info!("Starting Glypho...");
    let args = Args::parse();
    match args.commands {
        Cmds::StartServer { file, port } => {
            let shared_state = Arc::new(Mutex::new(InnerState::new(
                file.clone(),
                BytesMut::with_capacity(4096),
            )));

            let serve_dir = ServeDir::new(file.parent().unwrap());
            let router = Router::new()
                .route("/", get(root))
                .fallback_service(serve_dir)
                .route("/sse", get(event_handler))
                .with_state(shared_state);

            let port = port.unwrap_or(3030);

            let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
                .await
                .unwrap();

            let file_name = file.file_name().and_then(|fname| fname.to_str()).unwrap_or("unknown");
            tracing::info!("Serving {} at http://{}", file_name, listener.local_addr().unwrap());
            info!("Press Ctrl+C to stop the server");
            println!("");

            axum::serve(listener, router).await.unwrap();
        }
        Cmds::Compile {
            file: _,
            output_file: _,
        } => todo!(),
    }
    Ok(())
}

fn logger() {
    // if you want to see debug logs define the env var as GLYPHO=debug
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
            .with_level(is_debug)
        )
        .with(
            EnvFilter::try_new(format!("glypho={}", log_level))
                .expect("error in EnvFilter (Logger)"),
        )
        .init();

    println!("");
}