mod template;

use crate::template::TEMPLATE;
use async_watcher::{
    AsyncDebouncer, DebouncedEvent,
    notify::{self, RecommendedWatcher, RecursiveMode},
};
use axum::{
    Router,
    extract::State,
    response::{
        Html,
        sse::{Event, Sse},
    },
    routing::get,
};
use clap::{Parser, Subcommand};
use futures::{Stream, stream};
use handlebars::Handlebars;
use std::{
    collections::BTreeMap,
    convert::Infallible,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, atomic::AtomicBool},
    time::Duration,
};
use tokio_stream::StreamExt as _;
use tracing::*;

static CHANGED: AtomicBool = AtomicBool::new(false);

struct InnerState {
    pub file: PathBuf,
    pub rendered: String,
}

type AppState = Mutex<InnerState>;

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    commands: Cmds,
}

#[derive(Subcommand, Debug)]
enum Cmds {
    StartServer {
        #[arg(short, long)]
        file: PathBuf,
        #[arg(short, long)]
        port: Option<u16>,
    },

    Compile {
        #[arg(short, long)]
        file: PathBuf,
        output_file: PathBuf,
    },
}

pub async fn debounce_watch<P: AsRef<Path>>(
    path: P,
) -> Result<
    (
        tokio::sync::mpsc::Receiver<Result<Vec<DebouncedEvent>, Vec<notify::Error>>>,
        AsyncDebouncer<RecommendedWatcher>,
    ),
    Box<dyn std::error::Error>,
> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    let mut debouncer =
        AsyncDebouncer::new(Duration::from_secs(1), Some(Duration::from_secs(1)), tx).await?;

    // add the paths to the watcher
    debouncer
        .watcher()
        .watch(path.as_ref(), RecursiveMode::Recursive)?;

    Ok((rx, debouncer))
}

async fn event_handler(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    dbg!("Im'in");
    let binding = state.clone();
    let local_state = binding.lock().unwrap();

    let dir = local_state.file.parent().map(|s| s.to_owned()).unwrap();
    let filename = local_state.file.file_name().map(|s| s.to_owned());
    let f = format!(
        "{}/{}",
        dir.to_string_lossy().into_owned(),
        filename.clone().unwrap().into_string().unwrap()
    );

    tokio::spawn(async move {
        let (mut file_events, _debouncer) = debounce_watch(dir).await.unwrap();
        while let Some(events) = file_events.recv().await {
            match events {
                Ok(evs) => {
                    for ev in evs {
                        info!("{:?}", ev.path.to_str().unwrap());
                        if ev.path.file_name().map(|s| s.to_owned()) == filename {
                            CHANGED.store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                }
                Err(_) => todo!(),
            }
        }
    });

    let stream = stream::repeat_with(move || {
        let changed = CHANGED.load(std::sync::atomic::Ordering::Relaxed);
        CHANGED.store(false, std::sync::atomic::Ordering::Relaxed);
        if changed {
            let rendered = load_file(f.clone()).unwrap();
            Event::default().data(rendered.clone())
        } else {
            Event::default().data("false")
        }
    })
    .map(Ok)
    .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

fn load_file(file: impl AsRef<Path>) -> eyre::Result<String> {
    let mut hb = Handlebars::new();
    // register the template
    hb.register_template_string("template.html", TEMPLATE)?;
    let contents = fs::read_to_string(&file)?;
    let mut data = BTreeMap::new();
    let body = markdown::to_html(&contents.clone());
    data.insert("body".to_string(), body.clone());

    Ok(hb.render("template.html", &data)?)
}

async fn root(State(state): State<Arc<AppState>>) -> Html<String> {
    Html(state.lock().unwrap().rendered.clone())
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // create server-sent event

    match args.commands {
        Cmds::StartServer { file, port } => {
            let rendered = load_file(&file)?;

            let shared_state = Arc::new(Mutex::new(InnerState { file, rendered }));

            let router = Router::new()
                .route("/", get(root))
                .route("/sse", get(event_handler))
                .with_state(shared_state);

            let port = port.unwrap_or(3030);

            let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
                .await
                .unwrap();
            tracing::debug!("listening on {}", listener.local_addr().unwrap());
            axum::serve(listener, router).await.unwrap();
        }
        Cmds::Compile { file, output_file } => todo!(),
    }
    Ok(())
}
