use crate::{error::GlyphoError, template::TEMPLATE};
use async_watcher::{
    AsyncDebouncer, DebouncedEvent,
    notify::{self, RecommendedWatcher, RecursiveMode},
};
use axum::{
    extract::State,
    response::{
        Html,
        sse::{Event, Sse},
    },
};
use bytes::BytesMut;
use futures::{Stream, stream};
use handlebars::Handlebars;
use markdown::{CompileOptions, Constructs, Options, ParseOptions};
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

    // Add the paths to the watcher
    debouncer
        .watcher()
        .watch(path.as_ref(), RecursiveMode::Recursive)?;

    Ok((rx, debouncer))
}

pub async fn event_handler(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let binding = state.clone();
    let local_state = binding.lock().unwrap();

    let dir = local_state.file.parent().map(|s| s.to_owned()).unwrap();
    let filename = local_state.file.file_name().map(|s| s.to_owned());

    tokio::spawn(async move {
        let (mut file_events, _debouncer) = debounce_watch(dir).await.unwrap();
        while let Some(events) = file_events.recv().await {
            match events {
                Ok(evs) => {
                    for ev in evs {
                        info!("File {:?} changed", ev.path.to_str().unwrap());
                        if ev.path.file_name().map(|s| s.to_owned()) == filename {
                            CHANGED.store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                }
                Err(_) => panic!(),
            }
        }
    });

    let stream = stream::repeat_with(move || {
        let changed = CHANGED.load(std::sync::atomic::Ordering::Relaxed);
        CHANGED.store(false, std::sync::atomic::Ordering::Relaxed);
        if changed {
            let mut s = state.lock().unwrap();
            let html = s.reload_file().render().unwrap();
            Event::default().data(html)
        } else {
            Event::default().data("false")
        }
    })
    .map(Ok)
    .throttle(Duration::from_millis(100));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_millis(1000))
            .text("keep-alive-text"),
    )
}

pub async fn root(State(state): State<Arc<AppState>>) -> Html<String> {
    let html = state.lock().unwrap().render().unwrap();
    Html(html)
}

pub struct InnerState {
    file: PathBuf,
    rendered: BytesMut,
}

impl InnerState {
    pub fn new(file: PathBuf, rendered: BytesMut) -> Self {
        InnerState { file, rendered }
    }

    fn reload_file(&mut self) -> &mut Self {
        self.rendered.clear();
        let html = self.render().unwrap().as_bytes().into();
        self.rendered = html;
        self
    }

    fn render(&mut self) -> eyre::Result<String> {
        let mut hb = Handlebars::new();
        // register the template
        hb.register_template_string("template.html", TEMPLATE)?;

        let contents = match fs::read_to_string(&self.file) {
            Ok(c) => c,
            Err(err) => {
                dbg!(&err);
                String::from(format!("**Cannot Load the markdown file: {err:?}**"))
            }
        };
        let options = Options {
            parse: ParseOptions {
                constructs: Constructs {
                    code_indented: true,
                    gfm_table: true,
                    gfm_task_list_item: true,
                    attention: true,
                    frontmatter: true,
                    ..Constructs::gfm()
                },
                gfm_strikethrough_single_tilde: true,
                ..ParseOptions::default()
            },
            compile: CompileOptions {
                allow_dangerous_html: true,
                ..CompileOptions::gfm()
            },
            ..Options::default()
        };

        let mut data = BTreeMap::new();
        let body =
            markdown::to_html_with_options(&contents.clone(), &options).map_err(|message| {
                GlyphoError::MarkdownError {
                    place: message.place,
                    reason: message.reason,
                    rule_id: *message.rule_id,
                    m_source: *message.source,
                }
            })?;

        data.insert("body".to_string(), body.clone());
        Ok(hb.render("template.html", &data)?)
    }
}

type AppState = Mutex<InnerState>;
