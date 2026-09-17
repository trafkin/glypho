use std::path::{Path, PathBuf};
use std::sync::Arc;

use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig},
    schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
};

use crate::state::{AppState, activate_file, list_files, scan_markdown_files, track_file};

const INSTRUCTIONS: &str = "\
Glypho renders Markdown files in a live browser preview. To preview files the \
user just created or updated, first call detect_markdown_files to scan for \
Markdown files; it returns a numbered list you can ask the user to pick from. \
Then call open_markdown_files with the chosen paths in `files`, or with \
`open_all: true` to open everything proposed. Use list_tracked_files to see \
what is currently open and which file is active.";

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DetectMarkdownFilesArgs {
    /// Directory to scan for Markdown files. Defaults to the client's first
    /// MCP root when the client advertises roots, otherwise the server's
    /// current working directory.
    pub directory: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct OpenMarkdownFilesArgs {
    /// Markdown files to open and track.
    pub files: Option<Vec<String>>,
    /// Open every file proposed by the last `detect_markdown_files` call.
    pub open_all: Option<bool>,
}

#[derive(Clone)]
pub struct GlyphoMcpServer {
    state: Arc<AppState>,
}

#[tool_router]
impl GlyphoMcpServer {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    #[tool(
        description = "Scan a directory for Markdown files and propose them for opening in Glypho. Returns a question listing the files; call open_markdown_files with the chosen paths (or open_all: true) to open them."
    )]
    async fn detect_markdown_files(
        &self,
        Parameters(args): Parameters<DetectMarkdownFilesArgs>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let scan_root = match args.directory {
            Some(dir) => PathBuf::from(dir),
            None => match first_client_root(&ctx).await {
                Some(root) => root,
                None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            },
        };
        let scan_root = std::fs::canonicalize(&scan_root).unwrap_or(scan_root);

        let files = scan_markdown_files(&scan_root);

        {
            let mut s = self.state.lock().await;
            s.set_pending_markdown_files(files.clone());
        }

        if files.is_empty() {
            return Ok(CallToolResult::success(vec![ContentBlock::text(format!(
                "No Markdown files found under {}.",
                scan_root.display()
            ))]));
        }

        let mut message = format!(
            "Found {} Markdown file(s) under {}:\n",
            files.len(),
            scan_root.display()
        );
        for (index, file) in files.iter().enumerate() {
            message.push_str(&format!("{}. {}\n", index + 1, file.display()));
        }
        message.push_str(
            "\nWhich should I open in Glypho? Call open_markdown_files with the chosen \
             paths in `files`, or pass `open_all: true` to open all of them.",
        );

        Ok(CallToolResult::success(vec![ContentBlock::text(message)]))
    }

    #[tool(
        description = "Track and open Markdown files in Glypho's live preview. Provide `files` with explicit paths, or `open_all: true` to open every file proposed by the last detect_markdown_files call."
    )]
    async fn open_markdown_files(
        &self,
        Parameters(args): Parameters<OpenMarkdownFilesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let targets: Vec<PathBuf> = if args.open_all.unwrap_or(false) {
            let pending = {
                let s = self.state.lock().await;
                s.pending_markdown_files().to_vec()
            };
            if pending.is_empty() {
                return Ok(CallToolResult::error(vec![ContentBlock::text(
                    "No pending files. Call detect_markdown_files first to propose candidates.",
                )]));
            }
            pending
        } else {
            match args.files {
                Some(files) if !files.is_empty() => files.into_iter().map(PathBuf::from).collect(),
                _ => {
                    return Ok(CallToolResult::error(vec![ContentBlock::text(
                        "Provide `files` with at least one path, or set `open_all: true`.",
                    )]));
                }
            }
        };

        let mut opened: Vec<PathBuf> = Vec::new();
        let mut already_open: Vec<PathBuf> = Vec::new();
        let mut failed: Vec<String> = Vec::new();

        for target in targets {
            match validate_markdown_file(&target) {
                Ok(path) => {
                    if track_file(&self.state, path.clone()).await {
                        opened.push(path);
                    } else {
                        already_open.push(path);
                    }
                }
                Err(reason) => failed.push(format!("{}: {reason}", target.display())),
            }
        }

        if let Some(first) = opened.first().or(already_open.first()) {
            activate_file(&self.state, first.clone()).await;

            // Default glypho behavior: the preview belongs in the user's
            // browser. If no preview client is connected (tab closed or
            // server started detached), bring it back up. `--no-browser`
            // suppresses this via `should_open_preview`.
            let open_port = {
                let s = self.state.lock().await;
                if s.should_open_preview() {
                    s.listen_port()
                } else {
                    None
                }
            };
            if let Some(port) = open_port {
                let _ = open::that_detached(format!("http://127.0.0.1:{port}/"));
            }
        }

        let result = serde_json::json!({
            "opened": opened.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
            "already_open": already_open.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
            "failed": failed,
        });
        Ok(CallToolResult::success(vec![ContentBlock::text(
            result.to_string(),
        )]))
    }

    #[tool(description = "List the files Glypho is currently tracking and which one is active.")]
    async fn list_tracked_files(&self) -> Result<CallToolResult, McpError> {
        let (active_file, files) = list_files(&self.state).await;
        let result = serde_json::json!({
            "active_file": active_file.display().to_string(),
            "files": files.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
        });
        Ok(CallToolResult::success(vec![ContentBlock::text(
            result.to_string(),
        )]))
    }
}

#[tool_handler]
impl ServerHandler for GlyphoMcpServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("glypho", env!("CARGO_PKG_VERSION")))
            .with_instructions(INSTRUCTIONS)
    }
}

/// Resolve the client's first MCP root to a filesystem path.
///
/// MCP roots are deprecated by SEP-2577 but remain functional; clients that
/// still advertise them are the most accurate source for "the workspace the
/// agent is working in."
#[allow(deprecated)]
async fn first_client_root(ctx: &RequestContext<RoleServer>) -> Option<PathBuf> {
    let supports_roots = ctx
        .client_capabilities()
        .is_some_and(|caps| caps.roots.is_some());
    if !supports_roots {
        return None;
    }
    let roots = ctx.peer.list_roots().await.ok()?;
    let root = roots.roots.into_iter().next()?;
    let url = url::Url::parse(&root.uri).ok()?;
    url.to_file_path().ok()
}

/// Check that `path` is an existing, readable Markdown file and return its
/// canonical form.
fn validate_markdown_file(path: &Path) -> Result<PathBuf, String> {
    let is_markdown = path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"));
    if !is_markdown {
        return Err("not a Markdown file (expected .md or .markdown)".to_string());
    }
    match path.canonicalize() {
        Ok(canonical) if canonical.is_file() => Ok(canonical),
        Ok(_) => Err("not a regular file".to_string()),
        Err(err) => Err(format!("cannot read: {err}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn validate_accepts_md_file() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("note.md");
        std::fs::write(&file, "# Note").unwrap();

        let result = validate_markdown_file(&file);
        assert!(result.is_ok());
        assert!(result.unwrap().is_absolute());
    }

    #[test]
    fn validate_accepts_markdown_extension() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("note.markdown");
        std::fs::write(&file, "# Note").unwrap();

        assert!(validate_markdown_file(&file).is_ok());
    }

    #[test]
    fn validate_extension_is_case_insensitive() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("note.MD");
        std::fs::write(&file, "# Note").unwrap();

        assert!(validate_markdown_file(&file).is_ok());
    }

    #[test]
    fn validate_rejects_non_markdown_file() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("note.txt");
        std::fs::write(&file, "plain text").unwrap();

        let err = validate_markdown_file(&file).unwrap_err();
        assert!(err.contains("not a Markdown file"));
    }

    #[test]
    fn validate_rejects_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("missing.md");

        let err = validate_markdown_file(&file).unwrap_err();
        assert!(err.contains("cannot read"));
    }

    #[test]
    fn validate_rejects_directory_named_like_markdown() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path().join("docs.md");
        std::fs::create_dir(&dir).unwrap();

        let err = validate_markdown_file(&dir).unwrap_err();
        assert!(err.contains("not a regular file"));
    }
}
