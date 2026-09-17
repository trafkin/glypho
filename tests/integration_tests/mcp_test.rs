//! End-to-end tests for the MCP server.
//!
//! These tests spawn the real `glypho` binary with an isolated
//! `XDG_RUNTIME_DIR` (so the pidfile never collides with a user instance and
//! the test never enters client mode) and drive the Streamable HTTP MCP
//! endpoint over HTTP.

use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use reqwest::Client;
use serde_json::{Value, json};
use tempfile::TempDir;

const INITIALIZE_TIMEOUT: Duration = Duration::from_secs(15);

/// A spawned glypho server. Killed on drop.
struct GlyphoServer {
    child: Child,
    port: u16,
    /// Holds the temp dirs alive (working files + isolated runtime dir).
    _work_dir: TempDir,
    _runtime_dir: TempDir,
}

impl GlyphoServer {
    /// Start glypho serving `markdown` from a fresh temp file.
    async fn start(markdown: &str) -> Self {
        let work_dir = TempDir::new().unwrap();
        let runtime_dir = TempDir::new().unwrap();
        let file_path = work_dir.path().join("index.md");
        std::fs::write(&file_path, markdown).unwrap();

        let child = Command::new(env!("CARGO_BIN_EXE_glypho"))
            .arg("--no-browser")
            .arg("--port")
            .arg("0")
            .arg(&file_path)
            .env("XDG_RUNTIME_DIR", runtime_dir.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to spawn glypho");

        let pidfile = runtime_dir.path().join("glypho").join("running.pid");
        let deadline = Instant::now() + INITIALIZE_TIMEOUT;
        let port = loop {
            if let Ok(contents) = std::fs::read_to_string(&pidfile)
                && let Some(port) = parse_port(&contents)
            {
                // Confirm the server is actually answering before testing.
                if server_is_alive(port).await {
                    break port;
                }
            }
            assert!(
                Instant::now() < deadline,
                "glypho did not write a pidfile with a live port in time"
            );
            tokio::time::sleep(Duration::from_millis(100)).await;
        };

        Self {
            child,
            port,
            _work_dir: work_dir,
            _runtime_dir: runtime_dir,
        }
    }
}

impl Drop for GlyphoServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The pidfile is tiny TOML (`port = N\npid = N`); parse the port by hand to
/// avoid pulling a TOML parser into the test crate.
fn parse_port(pidfile: &str) -> Option<u16> {
    pidfile
        .lines()
        .find_map(|line| line.strip_prefix("port = ")?.trim().parse().ok())
}

async fn server_is_alive(port: u16) -> bool {
    reqwest::get(format!("http://localhost:{port}/"))
        .await
        .is_ok()
}

/// An initialized MCP session against a running server.
struct McpSession<'a> {
    client: &'a Client,
    port: u16,
    session_id: String,
    next_id: u64,
}

impl<'a> McpSession<'a> {
    async fn connect(client: &'a Client, port: u16) -> Self {
        let (headers, response) = mcp_post(client, port, None, &initialize_body(1)).await;
        assert!(
            response.get("result").is_some(),
            "initialize failed: {response}"
        );
        let session_id = headers
            .get("mcp-session-id")
            .expect("server did not return an mcp-session-id header")
            .to_str()
            .unwrap()
            .to_owned();

        let (status, _) = mcp_post_raw(
            client,
            port,
            Some(&session_id),
            &json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        )
        .await;
        assert_eq!(status, 202, "notifications/initialized rejected");

        Self {
            client,
            port,
            session_id,
            next_id: 2,
        }
    }

    async fn call_tool(&mut self, name: &str, arguments: Value) -> Value {
        let body = json!({
            "jsonrpc": "2.0",
            "id": self.next_id,
            "method": "tools/call",
            "params": { "name": name, "arguments": arguments },
        });
        self.next_id += 1;
        let (_, response) = mcp_post(self.client, self.port, Some(&self.session_id), &body).await;
        response
    }
}

fn initialize_body(id: u64) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": { "name": "glypho-test", "version": "0.0.0" },
        },
    })
}

/// POST a JSON-RPC message and parse the (SSE-framed) response body.
async fn mcp_post(
    client: &Client,
    port: u16,
    session_id: Option<&str>,
    body: &Value,
) -> (reqwest::header::HeaderMap, Value) {
    let (status, response) = mcp_post_raw(client, port, session_id, body).await;
    assert_eq!(status, 200, "unexpected status for {body}");
    let headers = response.headers().clone();
    let text = response.text().await.unwrap();
    // Streamable HTTP frames each JSON-RPC message as an SSE `data:` line;
    // keep the line that carries a JSON object.
    let data = text
        .lines()
        .filter_map(|line| line.strip_prefix("data: "))
        .find(|data| data.starts_with('{'))
        .unwrap_or_else(|| panic!("no JSON data line in SSE body: {text}"));
    let value: Value = serde_json::from_str(data).unwrap();
    (headers, value)
}

async fn mcp_post_raw(
    client: &Client,
    port: u16,
    session_id: Option<&str>,
    body: &Value,
) -> (reqwest::StatusCode, reqwest::Response) {
    let mut request = client
        .post(format!("http://localhost:{port}/mcp"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(body);
    if let Some(id) = session_id {
        request = request.header("Mcp-Session-Id", id);
    }
    let response = request.send().await.unwrap();
    (response.status(), response)
}

/// Extract the text content of a successful tool result.
fn tool_text(result: &Value) -> &str {
    result["result"]["content"][0]["text"]
        .as_str()
        .expect("tool result has no text content")
}

#[tokio::test]
async fn mcp_initialize_reports_glypho_server() {
    let server = GlyphoServer::start("# Hello").await;
    let client = Client::new();

    let (_, response) = mcp_post(&client, server.port, None, &initialize_body(1)).await;
    let result = &response["result"];
    assert_eq!(result["serverInfo"]["name"], "glypho");
    assert_eq!(result["serverInfo"]["version"], env!("CARGO_PKG_VERSION"));
    assert!(
        result["capabilities"].get("tools").is_some(),
        "tools capability missing: {result}"
    );
    assert!(
        result["instructions"]
            .as_str()
            .unwrap()
            .contains("detect_markdown_files")
    );
}

#[tokio::test]
async fn mcp_tools_list_exposes_exactly_the_three_tools() {
    let server = GlyphoServer::start("# Hello").await;
    let client = Client::new();
    let session = McpSession::connect(&client, server.port).await;

    let body = json!({
        "jsonrpc": "2.0",
        "id": session.next_id,
        "method": "tools/list",
        "params": {},
    });
    let (_, response) = mcp_post(&client, server.port, Some(&session.session_id), &body).await;

    let mut names: Vec<_> = response["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap().to_owned())
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec![
            "detect_markdown_files",
            "list_tracked_files",
            "open_markdown_files"
        ]
    );
}

#[tokio::test]
async fn mcp_open_all_without_prior_detect_is_a_tool_error() {
    let server = GlyphoServer::start("# Hello").await;
    let client = Client::new();
    let mut session = McpSession::connect(&client, server.port).await;

    let response = session
        .call_tool("open_markdown_files", json!({"open_all": true}))
        .await;

    assert_eq!(response["result"]["isError"], true);
    assert!(tool_text(&response).contains("detect_markdown_files first"));
}

#[tokio::test]
async fn mcp_open_with_empty_files_list_is_a_tool_error() {
    let server = GlyphoServer::start("# Hello").await;
    let client = Client::new();
    let mut session = McpSession::connect(&client, server.port).await;

    let response = session
        .call_tool("open_markdown_files", json!({"files": []}))
        .await;

    assert_eq!(response["result"]["isError"], true);
    assert!(tool_text(&response).contains("at least one path"));
}

#[tokio::test]
async fn mcp_open_with_missing_file_reports_failure() {
    let server = GlyphoServer::start("# Hello").await;
    let client = Client::new();
    let mut session = McpSession::connect(&client, server.port).await;

    let response = session
        .call_tool(
            "open_markdown_files",
            json!({"files": ["/definitely/not/here.md"]}),
        )
        .await;

    let parsed: Value = serde_json::from_str(tool_text(&response)).unwrap();
    assert!(parsed["opened"].as_array().unwrap().is_empty());
    assert_eq!(parsed["failed"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn mcp_full_question_flow_tracks_and_activates_files() {
    let server = GlyphoServer::start("# Hello").await;
    let scan_dir = TempDir::new().unwrap();
    let first = scan_dir.path().join("one.md");
    let second = scan_dir.path().join("two.md");
    std::fs::write(&first, "# One").unwrap();
    std::fs::write(&second, "# Two").unwrap();

    let client = Client::new();
    let mut session = McpSession::connect(&client, server.port).await;

    // 1. detect: returns the question and stores candidates
    let response = session
        .call_tool(
            "detect_markdown_files",
            json!({"directory": scan_dir.path().to_str().unwrap()}),
        )
        .await;
    let question = tool_text(&response);
    assert!(question.contains("Found 2 Markdown file(s)"));
    assert!(question.contains("open_all: true"));

    // 2. open_all: tracks both, activates the first
    let response = session
        .call_tool("open_markdown_files", json!({"open_all": true}))
        .await;
    let parsed: Value = serde_json::from_str(tool_text(&response)).unwrap();
    assert_eq!(parsed["opened"].as_array().unwrap().len(), 2);
    assert!(parsed["failed"].as_array().unwrap().is_empty());

    // 3. list: both files tracked, first one active
    let response = session.call_tool("list_tracked_files", json!({})).await;
    let parsed: Value = serde_json::from_str(tool_text(&response)).unwrap();
    let tracked: Vec<_> = parsed["files"].as_array().unwrap().clone();
    let canonical_first = std::fs::canonicalize(&first).unwrap();
    let canonical_second = std::fs::canonicalize(&second).unwrap();
    assert!(tracked.contains(&json!(canonical_first.to_string_lossy())));
    assert!(tracked.contains(&json!(canonical_second.to_string_lossy())));
    assert_eq!(
        parsed["active_file"].as_str().unwrap(),
        canonical_first.to_string_lossy()
    );
}

#[tokio::test]
async fn mcp_open_reports_already_open_for_tracked_file() {
    let server = GlyphoServer::start("# Hello").await;
    let client = Client::new();
    let mut session = McpSession::connect(&client, server.port).await;

    // The startup file is already tracked; opening it again by a different
    // path spelling must dedupe to already_open.
    let startup_file = server
        ._work_dir
        .path()
        .join("index.md")
        .to_string_lossy()
        .to_string();
    let response = session
        .call_tool("open_markdown_files", json!({"files": [startup_file]}))
        .await;

    let parsed: Value = serde_json::from_str(tool_text(&response)).unwrap();
    assert!(parsed["opened"].as_array().unwrap().is_empty());
    assert_eq!(parsed["already_open"].as_array().unwrap().len(), 1);
}
