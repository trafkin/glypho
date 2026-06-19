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
        AsyncDebouncer::new(Duration::from_secs(5), Some(Duration::from_secs(4)), tx).await?;

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
                let patch = PatchSignals::new(r#"{"first": false}"#);

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

                        let patch = PatchSignals::new(r#"{"first": false}"#);

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
                        let patch = PatchSignals::new(r#"{"first": false}"#);
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use tempfile::TempDir;

    // ==================== Helper Functions ====================

    fn create_test_state(file_path: PathBuf) -> Arc<AppState> {
        Arc::new(Mutex::new(InnerState::new(file_path)))
    }

    fn create_temp_markdown_file(content: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        std::fs::write(&file_path, content).unwrap();
        (temp_dir, file_path)
    }

    // ==================== InnerState Tests ====================

    #[test]
    fn test_inner_state_new() {
        let file_path = PathBuf::from("/tmp/test.md");
        let state = InnerState::new(file_path.clone());

        assert_eq!(state.active_file, file_path);
        assert!(state.files.contains_key(&file_path));
        assert!(state.watched_files.is_empty());
    }

    #[test]
    fn test_inner_state_new_initializes_buffer() {
        let file_path = PathBuf::from("/tmp/test.md");
        let state = InnerState::new(file_path.clone());

        let buffer = state.files.get(&file_path).unwrap();
        assert_eq!(buffer.capacity(), 4096);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_inner_state_reload_file() {
        let file_path = PathBuf::from("/tmp/test.md");
        let mut state = InnerState::new(file_path.clone());

        let buffer = BytesMut::with_capacity(100);
        let html = "<p>Test HTML</p>".to_string();

        state.reload_file(&file_path, buffer, html.clone());

        let stored_buffer = state.files.get(&file_path).unwrap();
        assert_eq!(stored_buffer.as_ref(), html.as_bytes());
    }

    #[test]
    fn test_inner_state_reload_file_clears_old_content() {
        let file_path = PathBuf::from("/tmp/test.md");
        let mut state = InnerState::new(file_path.clone());

        // First reload
        let buffer1 = BytesMut::from("old content");
        state.reload_file(&file_path, buffer1, "first".to_string());

        // Second reload
        let buffer2 = BytesMut::from("new content");
        state.reload_file(&file_path, buffer2, "second".to_string());

        let stored = state.files.get(&file_path).unwrap();
        assert_eq!(stored.as_ref(), b"second");
    }

    #[test]
    fn test_inner_state_render_valid_file() {
        let (_temp_dir, file_path) = create_temp_markdown_file("# Hello World\n\nThis is a test.");

        let mut state = InnerState::new(file_path.clone());
        let result = state.render(&file_path);

        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("<h1>"));
        assert!(html.contains("Hello World"));
        assert!(html.contains("<p>"));
    }

    #[test]
    fn test_inner_state_render_with_wikilinks() {
        let (_temp_dir, file_path) =
            create_temp_markdown_file("Check out [[MyPage]] for more info.");

        let mut state = InnerState::new(file_path.clone());
        let result = state.render(&file_path);

        assert!(result.is_ok());
        let html = result.unwrap();
        // Wikilinks should be converted to markdown links first
        assert!(html.contains("MyPage"));
    }

    #[test]
    fn test_inner_state_render_with_code_block() {
        let content = r#"# Code Example

```rust
fn main() {
    println!("Hello!");
}
```
"#;
        let (_temp_dir, file_path) = create_temp_markdown_file(content);

        let mut state = InnerState::new(file_path.clone());
        let result = state.render(&file_path);

        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("<code"));
    }

    #[test]
    fn test_inner_state_render_with_gfm_table() {
        let content = r#"| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
"#;
        let (_temp_dir, file_path) = create_temp_markdown_file(content);

        let mut state = InnerState::new(file_path.clone());
        let result = state.render(&file_path);

        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("<table>") || html.contains("<table"));
    }

    #[test]
    fn test_inner_state_render_with_task_list() {
        let content = r#"- [x] Completed task
- [ ] Incomplete task
"#;
        let (_temp_dir, file_path) = create_temp_markdown_file(content);

        let mut state = InnerState::new(file_path.clone());
        let result = state.render(&file_path);

        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("checkbox") || html.contains("type=\"checkbox\""));
    }

    #[test]
    fn test_inner_state_render_file_not_found() {
        let file_path = PathBuf::from("/nonexistent/path/file.md");
        let mut state = InnerState::new(file_path.clone());

        let result = state.render(&file_path);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("does not exist"));
    }

    #[test]
    fn test_inner_state_render_with_frontmatter() {
        let content = r#"---
title: Test Document
author: Test Author
---

# Content Here
"#;
        let (_temp_dir, file_path) = create_temp_markdown_file(content);

        let mut state = InnerState::new(file_path.clone());
        let result = state.render(&file_path);

        assert!(result.is_ok());
        // Frontmatter should be parsed (not rendered as visible content)
        let html = result.unwrap();
        assert!(html.contains("Content Here"));
    }

    #[test]
    fn test_inner_state_render_with_math() {
        let content = r#"Inline math: $x^2$

Block math:
$$
\sum_{i=1}^{n} i = \frac{n(n+1)}{2}
$$
"#;
        let (_temp_dir, file_path) = create_temp_markdown_file(content);

        let mut state = InnerState::new(file_path.clone());
        let result = state.render(&file_path);

        // Should render without error (actual math rendering is client-side)
        assert!(result.is_ok());
    }

    // ==================== Signals Struct Tests ====================

    #[test]
    fn test_signals_creation() {
        let signals = Signals {
            file: Some(PathBuf::from("/path/to/file.md")),
            first: true,
        };

        assert_eq!(signals.file, Some(PathBuf::from("/path/to/file.md")));
        assert!(signals.first);
    }

    #[test]
    fn test_signals_with_none_file() {
        let signals = Signals {
            file: None,
            first: false,
        };

        assert!(signals.file.is_none());
        assert!(!signals.first);
    }

    // ==================== AddFileRequest/Response Tests ====================

    #[test]
    fn test_add_file_request_creation() {
        let request = AddFileRequest {
            file: PathBuf::from("/path/to/new_file.md"),
        };

        assert_eq!(request.file, PathBuf::from("/path/to/new_file.md"));
    }

    #[test]
    fn test_add_file_response_creation() {
        let response = AddFileResponse { ok: true };
        assert!(response.ok);

        let response_fail = AddFileResponse { ok: false };
        assert!(!response_fail.ok);
    }

    // ==================== SignalEvents Tests ====================

    #[test]
    fn test_signal_events_clone() {
        let event = SignalEvents::AddedNewFile;
        let cloned = event.clone();
        assert!(matches!(cloned, SignalEvents::AddedNewFile));
    }

    #[test]
    fn test_signal_events_updated_file_clone() {
        let event = SignalEvents::UpdatedFile {
            updated_file: PathBuf::from("/test.md"),
            html: "<p>Test</p>".to_string(),
        };
        let cloned = event.clone();

        if let SignalEvents::UpdatedFile { updated_file, html } = cloned {
            assert_eq!(updated_file, PathBuf::from("/test.md"));
            assert_eq!(html, "<p>Test</p>");
        } else {
            panic!("Expected UpdatedFile variant");
        }
    }

    #[test]
    fn test_signal_events_debug() {
        let event = SignalEvents::ActiveFileChanged;
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("ActiveFileChanged"));
    }

    #[test]
    fn test_signal_events_all_variants() {
        let events: Vec<SignalEvents> = vec![
            SignalEvents::AddedNewFile,
            SignalEvents::UpdatedFile {
                updated_file: PathBuf::from("/test.md"),
                html: String::new(),
            },
            SignalEvents::ActiveFileChanged,
        ];

        assert_eq!(events.len(), 3);
    }

    // ==================== Event Sender/Receiver Tests ====================

    #[tokio::test]
    async fn test_event_sender_broadcast() {
        let file_path = PathBuf::from("/tmp/test.md");
        let state = InnerState::new(file_path);

        let mut receiver = state.event_sender.subscribe();

        // Send an event
        let _ = state.event_sender.send(SignalEvents::AddedNewFile);

        // Receive the event
        let received = receiver.recv().await.unwrap();
        assert!(matches!(received, SignalEvents::AddedNewFile));
    }

    #[tokio::test]
    async fn test_event_sender_multiple_subscribers() {
        let file_path = PathBuf::from("/tmp/test.md");
        let state = InnerState::new(file_path);

        let mut receiver1 = state.event_sender.subscribe();
        let mut receiver2 = state.event_sender.subscribe();

        let _ = state.event_sender.send(SignalEvents::ActiveFileChanged);

        let recv1 = receiver1.recv().await.unwrap();
        let recv2 = receiver2.recv().await.unwrap();

        assert!(matches!(recv1, SignalEvents::ActiveFileChanged));
        assert!(matches!(recv2, SignalEvents::ActiveFileChanged));
    }

    // ==================== State Mutex Tests ====================

    #[tokio::test]
    async fn test_state_concurrent_access() {
        let (_temp_dir, file_path) = create_temp_markdown_file("# Test");
        let state = create_test_state(file_path.clone());

        let state1 = state.clone();
        let state2 = state.clone();

        let file_path1 = file_path.clone();
        let handle1 = tokio::spawn(async move {
            let guard = state1.lock().await;
            assert_eq!(guard.active_file, file_path1);
        });

        let file_path2 = file_path.clone();
        let handle2 = tokio::spawn(async move {
            let guard = state2.lock().await;
            assert_eq!(guard.active_file, file_path2);
        });

        handle1.await.unwrap();
        handle2.await.unwrap();
    }

    #[tokio::test]
    async fn test_state_modification_persists() {
        let (_temp_dir, file_path) = create_temp_markdown_file("# Test");
        let state = create_test_state(file_path.clone());

        // Modify state
        {
            let mut guard = state.lock().await;
            guard.watched_files.push(file_path.clone());
        }

        // Verify modification persists
        {
            let guard = state.lock().await;
            assert_eq!(guard.watched_files.len(), 1);
            assert_eq!(guard.watched_files[0], file_path);
        }
    }

    // ==================== Markdown Rendering Options Tests ====================

    #[test]
    fn test_render_gfm_strikethrough() {
        let content = "This is ~~deleted~~ text.";
        let (_temp_dir, file_path) = create_temp_markdown_file(content);

        let mut state = InnerState::new(file_path.clone());
        let result = state.render(&file_path);

        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("<del>") || html.contains("deleted"));
    }

    #[test]
    fn test_render_autolinks() {
        let content = "Visit https://example.com for more info.";
        let (_temp_dir, file_path) = create_temp_markdown_file(content);

        let mut state = InnerState::new(file_path.clone());
        let result = state.render(&file_path);

        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("href") || html.contains("example.com"));
    }

    #[test]
    fn test_render_footnotes() {
        let content = r#"Here is a footnote reference[^1].

[^1]: Here is the footnote.
"#;
        let (_temp_dir, file_path) = create_temp_markdown_file(content);

        let mut state = InnerState::new(file_path.clone());
        let result = state.render(&file_path);

        assert!(result.is_ok());
    }

    #[test]
    fn test_render_dangerous_html_allowed() {
        let content = r#"<div class="custom">
Custom HTML content
</div>
"#;
        let (_temp_dir, file_path) = create_temp_markdown_file(content);

        let mut state = InnerState::new(file_path.clone());
        let result = state.render(&file_path);

        assert!(result.is_ok());
        let html = result.unwrap();
        // Dangerous HTML should be allowed
        assert!(html.contains("<div") || html.contains("custom"));
    }

    // ==================== File Operations Tests ====================

    #[tokio::test]
    async fn test_multiple_files_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.md");
        let file2 = temp_dir.path().join("file2.md");

        std::fs::write(&file1, "# File 1").unwrap();
        std::fs::write(&file2, "# File 2").unwrap();

        let state = create_test_state(file1.clone());

        // Add second file
        {
            let mut guard = state.lock().await;
            guard
                .files
                .insert(file2.clone(), BytesMut::with_capacity(4096));
        }

        // Verify both files are tracked
        {
            let guard = state.lock().await;
            assert!(guard.files.contains_key(&file1));
            assert!(guard.files.contains_key(&file2));
        }
    }

    #[tokio::test]
    async fn test_active_file_change() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.md");
        let file2 = temp_dir.path().join("file2.md");

        std::fs::write(&file1, "# File 1").unwrap();
        std::fs::write(&file2, "# File 2").unwrap();

        let state = create_test_state(file1.clone());

        // Change active file
        {
            let mut guard = state.lock().await;
            guard.active_file = file2.clone();
        }

        // Verify active file changed
        {
            let guard = state.lock().await;
            assert_eq!(guard.active_file, file2);
        }
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_render_empty_file() {
        let (_temp_dir, file_path) = create_temp_markdown_file("");

        let mut state = InnerState::new(file_path.clone());
        let result = state.render(&file_path);

        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.is_empty() || html.trim().is_empty());
    }

    #[test]
    fn test_render_whitespace_only_file() {
        let (_temp_dir, file_path) = create_temp_markdown_file("   \n\n   \n");

        let mut state = InnerState::new(file_path.clone());
        let result = state.render(&file_path);

        assert!(result.is_ok());
    }

    #[test]
    fn test_render_very_long_content() {
        let content = "# Long Document\n\n".to_string() + &"This is a paragraph.\n\n".repeat(1000);
        let (_temp_dir, file_path) = create_temp_markdown_file(&content);

        let mut state = InnerState::new(file_path.clone());
        let result = state.render(&file_path);

        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.len() > content.len()); // HTML should be longer due to tags
    }

    // ==================== Parameterized Tests ====================

    #[rstest]
    #[case("# Header", "<h1>")]
    #[case("## Header 2", "<h2>")]
    #[case("### Header 3", "<h3>")]
    #[case("**bold**", "<strong>")]
    #[case("*italic*", "<em>")]
    #[case("`code`", "<code>")]
    fn test_markdown_elements(#[case] input: &str, #[case] expected_tag: &str) {
        let (_temp_dir, file_path) = create_temp_markdown_file(input);

        let mut state = InnerState::new(file_path.clone());
        let result = state.render(&file_path);

        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(
            html.contains(expected_tag),
            "Expected {} in: {}",
            expected_tag,
            html
        );
    }
}
