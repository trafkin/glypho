use crate::{error::GlyphoError, template::TEMPLATE};
use async_watcher::{
    AsyncDebouncer, DebouncedEvent,
    notify::{self, RecommendedWatcher, RecursiveMode},
};
use asynk_strim::{Yielder, stream_fn};
use axum::{
    extract::State,
    response::{
        Html,
        sse::{Event, Sse},
    },
};
use bytes::BytesMut;
use datastar::{
    consts::ElementPatchMode,
    prelude::{ExecuteScript, PatchElements},
};
use eyre::bail;
use futures::Stream;
use markdown::{CompileOptions, Constructs, Options, ParseOptions};
use std::{
    convert::Infallible,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};
use tokio::sync::Mutex;
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

    let dir = {
        binding
            .lock()
            .await
            .file
            .parent()
            .map(|s| s.to_owned())
            .expect("Reading file error")
    };

    let stream = stream_fn(
        move |mut yielder: Yielder<Result<Event, Infallible>>| async move {
            CHANGED.store(false, std::sync::atomic::Ordering::Relaxed);

            let filename = { binding.lock().await.file.file_name().map(|s| s.to_owned()) };

            let (mut file_events, _debouncer) = debounce_watch(dir)
                .await
                .expect("Cannot get debouncer channel");

            while let Some(events) = file_events.recv().await {
                match events {
                    Ok(evs) => {
                        for ev in evs {
                            info!(
                                "File {:?} changed",
                                ev.path.to_str().expect("Path invalid unicode")
                            );
                            if ev.path.file_name().map(|s| s.to_owned()) == filename {
                                let mut s = state.lock().await;
                                let effect = s.reload_file().render();

                                let html: String = match effect {
                                    Ok(v) => v,
                                    Err(err) => {
                                        format!("Something weird happened:{}", err.to_string())
                                    }
                                };
                                let patch = PatchElements::new(html)
                                    .selector("article#markdown")
                                    .mode(ElementPatchMode::Inner);
                                let sse_event = patch.write_as_axum_sse_event();
                                yielder.yield_item(Ok(sse_event)).await;

                                let script = ExecuteScript::new(
                                    "Prism.highlightAllUnder(document.querySelector('article#markdown'));MathJax.typeset();",
                                );
                                let sse_event = script.write_as_axum_sse_event();
                                yielder.yield_item(Ok(sse_event)).await;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        },
    );

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_millis(1000))
            .text("keep-alive-text"),
    )
}

pub async fn root(State(_): State<Arc<AppState>>) -> Html<String> {
    let html = TEMPLATE.to_string();
    Html(html)
}

pub async fn init(State(state): State<Arc<AppState>>) -> Html<String> {
    let html = state
        .lock()
        .await
        .render()
        .expect("Cannot convert source markdown");
    Html(html)
}

pub struct InnerState {
    file: PathBuf,
    rendered: BytesMut,
}

impl InnerState {
    pub fn new(file: PathBuf, rendered: BytesMut) -> eyre::Result<Self> {
        let mut s = InnerState { file, rendered };
        s.render()?;
        Ok(s)
    }

    fn reload_file(&mut self) -> &mut Self {
        self.rendered.clear();
        let html = self
            .render()
            .expect("Cannot unlock mutex")
            .as_bytes()
            .into();
        self.rendered = html;
        self
    }

    fn render(&mut self) -> eyre::Result<String> {
        let content = match fs::read_to_string(&self.file) {
            Ok(c) => c,
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => bail!("The file or directory does not exist"),
                std::io::ErrorKind::PermissionDenied => {
                    bail!("Permission denied, insufficient permissions")
                }
                std::io::ErrorKind::ConnectionRefused => bail!("Connection refused by server"),
                std::io::ErrorKind::ConnectionReset => bail!("Connection was reset by peer"),
                std::io::ErrorKind::HostUnreachable => bail!("Host is unreachable"),
                std::io::ErrorKind::NetworkUnreachable => bail!("Network is unreachable"),
                std::io::ErrorKind::ConnectionAborted => {
                    bail!("Connection aborted, server closed the connection")
                }
                std::io::ErrorKind::NotConnected => bail!("Not connected to any server"),
                std::io::ErrorKind::AddrInUse => {
                    bail!("Address is already in use by another application")
                }
                std::io::ErrorKind::AddrNotAvailable => {
                    bail!("Address is not available or invalid")
                }
                std::io::ErrorKind::NetworkDown => bail!("Network interface is down"),
                std::io::ErrorKind::BrokenPipe => {
                    bail!("Broken pipe, connection closed unexpectedly")
                }
                std::io::ErrorKind::AlreadyExists => bail!("File or directory already exists"),
                std::io::ErrorKind::WouldBlock => bail!("Operation would block; try again later"),
                std::io::ErrorKind::NotADirectory => {
                    bail!("A file operation was attempted on a directory")
                }
                std::io::ErrorKind::IsADirectory => {
                    bail!("Directory operation was attempted on a file")
                }
                std::io::ErrorKind::DirectoryNotEmpty => bail!("Directory is not empty"),
                std::io::ErrorKind::ReadOnlyFilesystem => bail!("Read-only filesystem"),
                std::io::ErrorKind::StaleNetworkFileHandle => {
                    bail!("Stale network file handle, refresh or invalidate")
                }
                std::io::ErrorKind::InvalidInput => bail!("Invalid input provided"),
                std::io::ErrorKind::InvalidData => bail!("Corrupted data encountered"),
                std::io::ErrorKind::TimedOut => bail!("Operation timed out"),
                std::io::ErrorKind::WriteZero => bail!("No bytes were written"),
                std::io::ErrorKind::StorageFull => bail!("Storage is full"),
                std::io::ErrorKind::NotSeekable => bail!("File or stream is not seekable"),
                std::io::ErrorKind::QuotaExceeded => bail!("User quota exceeded"),
                std::io::ErrorKind::FileTooLarge => bail!("File exceeds filesystem limits"),
                std::io::ErrorKind::ResourceBusy => bail!("Resource is busy, try again later"),
                std::io::ErrorKind::ExecutableFileBusy => bail!("Executable file is busy"),
                std::io::ErrorKind::Deadlock => bail!("Deadlock detected"),
                std::io::ErrorKind::CrossesDevices => bail!("Operation crosses device boundaries"),
                std::io::ErrorKind::TooManyLinks => bail!("Too many links in path"),
                std::io::ErrorKind::InvalidFilename => bail!("Invalid filename or directory name"),
                std::io::ErrorKind::ArgumentListTooLong => bail!("Argument list is too long"),
                std::io::ErrorKind::Interrupted => bail!("Operation was interrupted"),
                std::io::ErrorKind::Unsupported => {
                    bail!("Operation not supported on this platform")
                }
                std::io::ErrorKind::UnexpectedEof => bail!("Unexpected end of file"),
                std::io::ErrorKind::OutOfMemory => bail!("Out of memory"),
                std::io::ErrorKind::Other => bail!("An unspecified I/O error occurred"),
                _ => bail!("An unknown error occurred: {:?}", err),
            },
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

        let body =
            markdown::to_html_with_options(&content.clone(), &options).map_err(|message| {
                GlyphoError::MarkdownError {
                    place: message.place,
                    reason: message.reason,
                    rule_id: *message.rule_id,
                    m_source: *message.source,
                }
            })?;

        Ok(body)
    }
}

type AppState = Mutex<InnerState>;
