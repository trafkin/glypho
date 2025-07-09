mod template;

use clap::{Parser, Subcommand};
use futures_util::{Stream, StreamExt};
use handlebars::Handlebars;
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{DebounceEventResult, DebouncedEvent, Debouncer, new_debouncer};
use std::{
    collections::BTreeMap,
    convert::Infallible,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::fs;
use tokio_stream::wrappers::{ReceiverStream, UnboundedReceiverStream};
use tracing::*;
use warp::{Filter, reply::Reply, sse::Event};

use crate::template::TEMPLATE;

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

pub fn debounce_watch<P: AsRef<Path>>(
    path: P,
) -> Result<
    (
        tokio::sync::mpsc::UnboundedReceiver<Vec<DebouncedEvent>>,
        Debouncer<RecommendedWatcher>,
    ),
    Box<dyn std::error::Error>,
> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    let mut debouncer =
        new_debouncer(
            Duration::from_secs(2),
            move |res: DebounceEventResult| match res {
                Ok(events) => tx.send(events).unwrap(),
                Err(e) => println!("Error {:?}", e),
            },
        )
        .unwrap();

    // add the paths to the watcher
    debouncer
        .watcher()
        .watch(path.as_ref(), RecursiveMode::Recursive)?;

    Ok((rx, debouncer))
}

fn change_event(counter: usize) -> Result<Event, Infallible> {
    Ok(warp::sse::Event::default().data(format!("{counter}")))
}

fn event_handler<P: AsRef<Path>>(
    path: P,
) -> impl Stream<Item = Result<Event, warp::Error>> + Send + 'static {
    dbg!("Im'in");

    let dir = path.as_ref().parent().unwrap();

    let (file_events, _debouncer) = debounce_watch(dir).unwrap();

    let file_events = UnboundedReceiverStream::new(file_events);
    // I left this only to compile, the above prints, but the next doesn't work
    let event_stream = file_events.map(|events| {
        let size = events.len();
        for ev in events {
            info!("{:?}", ev.path.to_str().unwrap())
        }
        Ok(warp::sse::Event::default().data(format!("{size}")))
    });
    event_stream
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let mut hb = Handlebars::new();
    // register the template
    hb.register_template_string("template.html", TEMPLATE)?;

    // create server-sent event

    match args.commands {
        Cmds::StartServer { file, port } => {
            let contents = fs::read_to_string(&file).await?;
            let mut data = BTreeMap::new();
            let body = markdown::to_html(&contents.clone());
            data.insert("body".to_string(), body.clone());

            let render = hb
                .render("template.html", &data)
                .unwrap_or_else(|err| err.to_string());

            let route = warp::path::end().map(move || warp::reply::html(render.clone()));

            let sse = warp::path("reload")
                .and(warp::get())
                .and(warp::any().map(move || file.clone()))
                .map(|file| {
                    let stream = event_handler(file);
                    warp::sse::reply(warp::sse::keep_alive().stream(stream))
                });

            let port = port.unwrap_or(3030);

            warp::serve(route.or(sse)).run(([127, 0, 0, 1], port)).await
        }
        Cmds::Compile { file, output_file } => todo!(),
    }
    Ok(())
}
