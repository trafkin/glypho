use crate::{error::GlyphoError, template::TEMPLATE, wikilinks::wikilinks_to_markdown};
use async_watcher::{
    AsyncDebouncer, DebouncedEvent,
    notify::{self, RecommendedWatcher, RecursiveMode},
};
use asynk_strim::{Yielder, stream_fn};
use axum::{
    Json,
    extract::{self, State},
    response::{
        Html, IntoResponse,
        sse::{Event, Sse},
    },
};
use bytes::BytesMut;
use datastar::{
    axum::ReadSignals,
    consts::ElementPatchMode,
    prelude::{ExecuteScript, PatchElements, PatchSignals},
};
use eyre::bail;
use futures::FutureExt;
use markdown::{CompileOptions, Constructs, Options, ParseOptions};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    convert::Infallible,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::sync::{
    Mutex, MutexGuard,
    broadcast::{self, Sender},
};

use tracing::*;

#[derive(Serialize, Deserialize)]
pub struct Signals {
    pub file: Option<PathBuf>,
    pub first: bool,
}

#[derive(Serialize, Deserialize)]
pub struct AddFileRequest {
    pub file: PathBuf,
}

#[derive(Serialize)]
pub struct AddFileResponse {
    pub ok: bool,
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
        AsyncDebouncer::new(Duration::from_secs(10), Some(Duration::from_secs(9)), tx).await?;

    // Add the paths to the watcher
    debouncer
        .watcher()
        .watch(path.as_ref(), RecursiveMode::Recursive)?;

    Ok((rx, debouncer))
}

pub async fn watch_file(file: PathBuf, state: Arc<AppState>) {
    let local_state = state.clone();

    if !local_state
        .lock()
        .await
        .watched_files
        .iter()
        .any(|v| v == &file)
    {
        local_state.lock().await.watched_files.push(file.clone());
        debug!("file not watched");
        tokio::spawn(async move {
            let dir = file.parent().expect("Reading file error");
            let (mut file_events, _debouncer) = debounce_watch(dir)
                .await
                .expect("Cannot get debouncer channel");
            while let Some(file_watcher_events) = file_events.recv().await {
                if let Ok(evs) = file_watcher_events {
                    for ev in evs {
                        debug!(
                            "File {:?} changed",
                            ev.path.to_str().expect("Path invalid unicode")
                        );
                        if ev
                            .path
                            .file_name()
                            .zip(file.file_name())
                            .map(|(f1, f2)| f1 == f2)
                            .unwrap_or(false)
                        {
                            // let mut s = local_state.lock().await;
                            let buffer = {
                                local_state
                                    .lock()
                                    .await
                                    .files
                                    .get(&file)
                                    .map(|b| b.to_owned())
                            };
                            let rendered = { local_state.lock().await.render(&file) };

                            {
                                let mut s = local_state.lock().await;
                                if let Some((html, buf)) = rendered.as_ref().ok().zip(buffer) {
                                    s.reload_file(&file, buf.to_owned(), html.clone());
                                };
                            }

                            // s.reload_file(file, buffer.unwrap(), rendered);

                            let html: String = match rendered {
                                Ok(v) => v,
                                Err(err) => {
                                    format!("Something weird happened:{}", err)
                                }
                            };

                            let _ = {
                                local_state.lock().await.event_sender.send(
                                    SignalEvents::UpdatedFile {
                                        updated_file: file.clone(),
                                        html,
                                    },
                                )
                            };
                        }
                    }
                }
            }
        });
    }
}

pub async fn add_file(
    State(state): State<Arc<AppState>>,
    extract::Json(file_request): extract::Json<AddFileRequest>,
) -> impl IntoResponse {
    let buffer = BytesMut::with_capacity(4096);
    let file = file_request.file.clone();
    state
        .lock()
        .then(|mut s: MutexGuard<InnerState>| async move {
            s.files.insert(file_request.file.clone(), buffer);
        })
        .await;

    watch_file(file, state.clone()).await;

    let _ = state
        .lock()
        .await
        .event_sender
        .send(SignalEvents::AddedNewFile);

    Json(AddFileResponse { ok: true })
}

pub async fn change_active(
    State(state): State<Arc<AppState>>,
    ReadSignals(signals): ReadSignals<Signals>,
) -> impl IntoResponse {
    state
        .lock()
        .then(|mut s: MutexGuard<InnerState>| async move {
            match signals.file {
                Some(ref f) => {
                    s.active_file = f.clone();
                }
                None => {
                    debug!("file not found");
                }
            };
            let _ = s.event_sender.send(SignalEvents::ActiveFileChanged);
        })
        .await;
    Json(AddFileResponse { ok: true })
}

pub async fn event_handler(
    State(state): State<Arc<AppState>>,
    ReadSignals(signals): ReadSignals<Signals>,
) -> impl IntoResponse {
    // stream over broadcast events

    let stream = stream_fn(
        move |mut yielder: Yielder<Result<Event, Infallible>>| async move {
            // render and start listening file changes
            // todo!();
            let local_state = state.clone();

            if signals.first {
                // let mut s = local_state.lock().await;
                let file = { local_state.lock().await.active_file.clone() };

                let buffer = {
                    local_state
                        .lock()
                        .await
                        .files
                        .get(&file)
                        .map(|b| b.to_owned())
                };
                let rendered = { local_state.lock().await.render(&file) };

                {
                    let mut s = local_state.lock().await;
                    if let Some((html, buf)) = rendered.as_ref().ok().zip(buffer) {
                        s.reload_file(&file, buf.to_owned(), html.clone());
                    };
                }

                // s.reload_file(file, buffer.unwrap(), rendered);

                let html: String = match rendered {
                    Ok(v) => v,
                    Err(err) => {
                        format!("Something weird happened:{}", err)
                    }
                };
                let patch = PatchSignals::new(r#"{{"first": false}}"#);

                let sse_event = patch.write_as_axum_sse_event();
                yielder.yield_item(Ok(sse_event)).await;

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
            let len = { state.lock().await.watched_files.len() };
            if len > 1 {
                let mut buttons = vec![];
                for path in state.lock().await.watched_files.iter() {
                    let filename = path
                        .file_name()
                        .unwrap_or_default()
                        .to_os_string()
                        .into_string()
                        .unwrap_or_default();
                    let string_path = path
                        .clone()
                        .into_os_string()
                        .into_string()
                        .unwrap_or_default();

                    let button = format!(
                        "<button id ='{string_path}' class='rounded-md px-5 py-2.5 leading-5 font-semibold' data-on:click=\"$file = '{string_path}';@get('/update')\" >{filename}</button><br />"
                    );
                    buttons.push(button);
                }

                let patch = PatchElements::new(buttons.join("\n"))
                    .selector("nav#navbar")
                    .mode(ElementPatchMode::Replace);
                let sse_event = patch.write_as_axum_sse_event();
                yielder.yield_item(Ok(sse_event)).await;
            }

            let mut events = { state.lock().await.event_sender.subscribe() };

            while let Ok(signal_events) = events.recv().await {
                match signal_events {
                    SignalEvents::AddedNewFile => {
                        let mut buttons = vec![];
                        for path in state.lock().await.watched_files.iter() {
                            let filename = path
                                .file_name()
                                .unwrap_or_default()
                                .to_os_string()
                                .into_string()
                                .unwrap();
                            let string_path = path.clone().into_os_string().into_string().unwrap();

                            let button = format!(
                                "<button id ='{string_path}' class='rounded-md px-5 py-2.5 leading-5 font-semibold' data-on:click=\"$file = '{string_path}';@get('/update')\">{filename}</button><br />"
                            );
                            buttons.push(button);
                        }
                        let html = buttons.join("\n");
                        let patch = PatchElements::new(html)
                            .selector("nav#navbar")
                            .mode(ElementPatchMode::Inner);
                        let sse_event = patch.write_as_axum_sse_event();
                        yielder.yield_item(Ok(sse_event)).await;
                    }

                    SignalEvents::UpdatedFile { updated_file, html } => {
                        // from inotify
                        // send html signals
                        let active = { local_state.lock().await.active_file.clone() };
                        if active == updated_file {
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

                        let patch = PatchSignals::new(r#"{{"first": false}}"#);

                        let sse_event = patch.write_as_axum_sse_event();
                        yielder.yield_item(Ok(sse_event)).await;
                    }
                    SignalEvents::ActiveFileChanged => {
                        // signal active file
                        //
                        let file = { local_state.lock().await.active_file.clone() };

                        let buffer = {
                            local_state
                                .lock()
                                .await
                                .files
                                .get(&file)
                                .map(|b| b.to_owned())
                        };
                        let rendered = { local_state.lock().await.render(&file) };

                        {
                            let mut s = local_state.lock().await;
                            if let Some((html, buf)) = rendered.as_ref().ok().zip(buffer) {
                                s.reload_file(&file, buf.to_owned(), html.clone());
                            };
                        }

                        // s.reload_file(file, buffer.unwrap(), rendered);

                        let html: String = match rendered {
                            Ok(v) => v,
                            Err(err) => {
                                format!("Something weird happened:{}", err)
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

                        let script = ExecuteScript::new(
                            "Prism.highlightAllUnder(document.querySelector('article#markdown'));MathJax.typeset();",
                        );
                        let sse_event = script.write_as_axum_sse_event();
                        yielder.yield_item(Ok(sse_event)).await;
                        let patch = PatchSignals::new(r#"{{"first": false}}"#);
                        let sse_event = patch.write_as_axum_sse_event();
                        yielder.yield_item(Ok(sse_event)).await;
                    }
                };
            }
        },
    );

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_millis(100))
            .text("keep-alive-text"),
    )
}

pub async fn root(State(state): State<Arc<AppState>>) -> Html<String> {
    let local_state = state.clone();

    let file = { local_state.lock().await.active_file.clone() };

    watch_file(file, state.clone()).await;

    let html = TEMPLATE.to_string();
    Html(html)
}

#[derive(Clone, Debug)]
pub enum SignalEvents {
    // WatchFile { file: PathBuf },
    AddedNewFile,
    UpdatedFile { updated_file: PathBuf, html: String },
    ActiveFileChanged,
}

pub struct InnerState {
    files: BTreeMap<PathBuf, BytesMut>,
    active_file: PathBuf,
    event_sender: Sender<SignalEvents>,
    // event_reciever: Receiver<SignalEvents>,
    watched_files: Vec<PathBuf>,
}

impl InnerState {
    pub fn new(first_file: PathBuf) -> Self {
        let mut files: BTreeMap<PathBuf, BytesMut> = BTreeMap::new();
        let buffer = BytesMut::with_capacity(4096);
        let (event_sender, _) = broadcast::channel(32);

        files.insert(first_file.clone(), buffer);
        InnerState {
            files,
            active_file: first_file,
            event_sender,
            watched_files: vec![],
        }
    }

    fn reload_file(&mut self, file: &Path, mut buffer: BytesMut, html: String) -> &mut Self {
        buffer.clear();
        buffer = html.as_bytes().into();

        self.files.insert(file.to_path_buf(), buffer);
        self
    }

    fn render(&mut self, file: &PathBuf) -> eyre::Result<String> {
        let (file, _buffer) = self.files.get_key_value(file).unzip();
        let content = match fs::read_to_string(file.expect("file not being tracked")) {
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
                    gfm_footnote_definition: true,
                    autolink: true,
                    gfm_autolink_literal: true,
                    ..Constructs::gfm()
                },
                gfm_strikethrough_single_tilde: true,
                ..ParseOptions::default()
            },
            compile: CompileOptions {
                allow_dangerous_html: true,

                ..CompileOptions::gfm()
            },
        };

        let with_wikilinks = wikilinks_to_markdown(&content);

        let body =
            markdown::to_html_with_options(&with_wikilinks, &options).map_err(|message| {
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
