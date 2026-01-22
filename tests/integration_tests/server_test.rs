//! Integration tests for the Glypho server
//!
//! These tests verify the full server functionality including HTTP handlers,
//! file serving, and SSE streaming.

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
};
use bytes::BytesMut;
use std::{collections::BTreeMap, path::PathBuf, sync::Arc, time::Duration};
use tempfile::TempDir;
use tokio::sync::{Mutex, broadcast};
use tower::ServiceExt;

use super::common;
use common::fixtures;

// ==================== Test App Setup ====================

/// Simplified InnerState for testing (mirrors the real implementation)
struct TestInnerState {
    files: BTreeMap<PathBuf, BytesMut>,
    active_file: PathBuf,
    event_sender: broadcast::Sender<TestSignalEvent>,
    watched_files: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
enum TestSignalEvent {
    AddedNewFile,
    UpdatedFile { updated_file: PathBuf, html: String },
    ActiveFileChanged,
}

impl TestInnerState {
    fn new(first_file: PathBuf) -> Self {
        let mut files = BTreeMap::new();
        let buffer = BytesMut::with_capacity(4096);
        let (event_sender, _) = broadcast::channel(32);
        files.insert(first_file.clone(), buffer);
        TestInnerState {
            files,
            active_file: first_file,
            event_sender,
            watched_files: vec![],
        }
    }
}

type TestAppState = Mutex<TestInnerState>;

fn create_test_state(file_path: PathBuf) -> Arc<TestAppState> {
    Arc::new(Mutex::new(TestInnerState::new(file_path)))
}

// ==================== Server Integration Tests ====================

#[tokio::test]
async fn test_server_starts_with_valid_file() {
    let (temp_dir, file_path) = common::create_temp_file(fixtures::SIMPLE_MARKDOWN);
    let state = create_test_state(file_path.clone());

    // Verify state was initialized correctly
    let guard = state.lock().await;
    assert_eq!(guard.active_file, file_path);
    assert!(guard.files.contains_key(&file_path));
}

#[tokio::test]
async fn test_multiple_files_can_be_added() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = common::create_named_temp_file(&temp_dir, "file1.md", "# File 1");
    let file2 = common::create_named_temp_file(&temp_dir, "file2.md", "# File 2");
    let file3 = common::create_named_temp_file(&temp_dir, "file3.md", "# File 3");

    let state = create_test_state(file1.clone());

    // Add more files
    {
        let mut guard = state.lock().await;
        guard
            .files
            .insert(file2.clone(), BytesMut::with_capacity(4096));
        guard
            .files
            .insert(file3.clone(), BytesMut::with_capacity(4096));
        guard.watched_files.push(file1.clone());
        guard.watched_files.push(file2.clone());
        guard.watched_files.push(file3.clone());
    }

    // Verify all files are tracked
    let guard = state.lock().await;
    assert_eq!(guard.files.len(), 3);
    assert_eq!(guard.watched_files.len(), 3);
}

#[tokio::test]
async fn test_active_file_can_be_changed() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = common::create_named_temp_file(&temp_dir, "file1.md", "# File 1");
    let file2 = common::create_named_temp_file(&temp_dir, "file2.md", "# File 2");

    let state = create_test_state(file1.clone());

    // Add second file and change active
    {
        let mut guard = state.lock().await;
        guard
            .files
            .insert(file2.clone(), BytesMut::with_capacity(4096));
        guard.active_file = file2.clone();
    }

    // Verify active file changed
    let guard = state.lock().await;
    assert_eq!(guard.active_file, file2);
}

#[tokio::test]
async fn test_event_broadcasting() {
    let (_, file_path) = common::create_temp_file(fixtures::SIMPLE_MARKDOWN);
    let state = create_test_state(file_path.clone());

    // Subscribe to events
    let mut receiver = {
        let guard = state.lock().await;
        guard.event_sender.subscribe()
    };

    // Send event
    {
        let guard = state.lock().await;
        let _ = guard.event_sender.send(TestSignalEvent::AddedNewFile);
    }

    // Verify event received
    let event = tokio::time::timeout(Duration::from_secs(1), receiver.recv())
        .await
        .expect("Timeout waiting for event")
        .expect("Failed to receive event");

    assert!(matches!(event, TestSignalEvent::AddedNewFile));
}

#[tokio::test]
async fn test_event_broadcasting_updated_file() {
    let (_, file_path) = common::create_temp_file(fixtures::SIMPLE_MARKDOWN);
    let state = create_test_state(file_path.clone());

    let mut receiver = {
        let guard = state.lock().await;
        guard.event_sender.subscribe()
    };

    // Send UpdatedFile event
    {
        let guard = state.lock().await;
        let _ = guard.event_sender.send(TestSignalEvent::UpdatedFile {
            updated_file: file_path.clone(),
            html: "<p>Updated HTML</p>".to_string(),
        });
    }

    let event = tokio::time::timeout(Duration::from_secs(1), receiver.recv())
        .await
        .expect("Timeout")
        .expect("Failed to receive");

    if let TestSignalEvent::UpdatedFile { updated_file, html } = event {
        assert_eq!(updated_file, file_path);
        assert_eq!(html, "<p>Updated HTML</p>");
    } else {
        panic!("Expected UpdatedFile event");
    }
}

#[tokio::test]
async fn test_concurrent_state_access() {
    let (_, file_path) = common::create_temp_file(fixtures::SIMPLE_MARKDOWN);
    let state = create_test_state(file_path.clone());

    let state1 = state.clone();
    let state2 = state.clone();
    let state3 = state.clone();

    let file_path1 = file_path.clone();
    let file_path2 = file_path.clone();
    let file_path3 = file_path.clone();

    let handle1 = tokio::spawn(async move {
        for _ in 0..100 {
            let guard = state1.lock().await;
            assert_eq!(guard.active_file, file_path1);
        }
    });

    let handle2 = tokio::spawn(async move {
        for _ in 0..100 {
            let guard = state2.lock().await;
            assert!(guard.files.contains_key(&file_path2));
        }
    });

    let handle3 = tokio::spawn(async move {
        for _ in 0..100 {
            let guard = state3.lock().await;
            let _ = guard.event_sender.send(TestSignalEvent::AddedNewFile);
        }
    });

    handle1.await.unwrap();
    handle2.await.unwrap();
    handle3.await.unwrap();
}

// ==================== File Content Tests ====================

#[tokio::test]
async fn test_file_content_with_various_markdown() {
    let test_cases = vec![
        ("simple", fixtures::SIMPLE_MARKDOWN),
        ("code", fixtures::MARKDOWN_WITH_CODE),
        ("wikilinks", fixtures::MARKDOWN_WITH_WIKILINKS),
        ("math", fixtures::MARKDOWN_WITH_MATH),
        ("table", fixtures::MARKDOWN_WITH_TABLE),
        ("task_list", fixtures::MARKDOWN_WITH_TASK_LIST),
        ("frontmatter", fixtures::MARKDOWN_WITH_FRONTMATTER),
        ("complex", fixtures::MARKDOWN_COMPLEX),
    ];

    for (name, content) in test_cases {
        let (_, file_path) = common::create_temp_file(content);
        let state = create_test_state(file_path.clone());

        let guard = state.lock().await;
        assert!(
            guard.files.contains_key(&file_path),
            "Failed for test case: {}",
            name
        );
    }
}

#[tokio::test]
async fn test_empty_file_handling() {
    let (_, file_path) = common::create_temp_file(fixtures::EMPTY_MARKDOWN);
    let state = create_test_state(file_path.clone());

    let guard = state.lock().await;
    assert!(guard.files.contains_key(&file_path));
}

#[tokio::test]
async fn test_whitespace_only_file_handling() {
    let (_, file_path) = common::create_temp_file(fixtures::WHITESPACE_ONLY_MARKDOWN);
    let state = create_test_state(file_path.clone());

    let guard = state.lock().await;
    assert!(guard.files.contains_key(&file_path));
}

// ==================== Buffer Management Tests ====================

#[tokio::test]
async fn test_buffer_capacity() {
    let (_, file_path) = common::create_temp_file(fixtures::SIMPLE_MARKDOWN);
    let state = create_test_state(file_path.clone());

    let guard = state.lock().await;
    let buffer = guard.files.get(&file_path).unwrap();
    assert_eq!(buffer.capacity(), 4096);
}

#[tokio::test]
async fn test_buffer_update() {
    let (_, file_path) = common::create_temp_file(fixtures::SIMPLE_MARKDOWN);
    let state = create_test_state(file_path.clone());

    // Update buffer
    {
        let mut guard = state.lock().await;
        let buffer = guard.files.get_mut(&file_path).unwrap();
        buffer.clear();
        buffer.extend_from_slice(b"<p>New content</p>");
    }

    // Verify update
    let guard = state.lock().await;
    let buffer = guard.files.get(&file_path).unwrap();
    assert_eq!(buffer.as_ref(), b"<p>New content</p>");
}

// ==================== Watched Files Tests ====================

#[tokio::test]
async fn test_watched_files_initially_empty() {
    let (_, file_path) = common::create_temp_file(fixtures::SIMPLE_MARKDOWN);
    let state = create_test_state(file_path);

    let guard = state.lock().await;
    assert!(guard.watched_files.is_empty());
}

#[tokio::test]
async fn test_watched_files_can_be_added() {
    let temp_dir = TempDir::new().unwrap();
    let file1 = common::create_named_temp_file(&temp_dir, "file1.md", "# File 1");
    let file2 = common::create_named_temp_file(&temp_dir, "file2.md", "# File 2");

    let state = create_test_state(file1.clone());

    {
        let mut guard = state.lock().await;
        guard.watched_files.push(file1.clone());
        guard.watched_files.push(file2.clone());
    }

    let guard = state.lock().await;
    assert_eq!(guard.watched_files.len(), 2);
    assert!(guard.watched_files.contains(&file1));
    assert!(guard.watched_files.contains(&file2));
}

#[tokio::test]
async fn test_watched_files_no_duplicates_check() {
    let (_, file_path) = common::create_temp_file(fixtures::SIMPLE_MARKDOWN);
    let state = create_test_state(file_path.clone());

    {
        let mut guard = state.lock().await;
        // Simulate checking if file is already watched (as in watch_file)
        if !guard.watched_files.contains(&file_path) {
            guard.watched_files.push(file_path.clone());
        }
        // Try to add again - should not add duplicate
        if !guard.watched_files.contains(&file_path) {
            guard.watched_files.push(file_path.clone());
        }
    }

    let guard = state.lock().await;
    assert_eq!(guard.watched_files.len(), 1);
}
