//! Common test utilities and helpers
//!
//! This module provides shared functionality for integration tests.

use axum::{
    Router,
    routing::{get, post},
};
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;

/// Test server configuration
pub struct TestServer {
    pub temp_dir: TempDir,
    pub file_path: PathBuf,
    pub app: Router,
}

/// Create a temporary markdown file with the given content
pub fn create_temp_file(content: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.md");
    std::fs::write(&file_path, content).expect("Failed to write test file");
    (temp_dir, file_path)
}

/// Create a temporary markdown file with a specific name
pub fn create_named_temp_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let file_path = dir.path().join(name);
    std::fs::write(&file_path, content).expect("Failed to write test file");
    file_path
}

/// Sample markdown content for testing
pub mod fixtures {
    pub const SIMPLE_MARKDOWN: &str = r#"# Hello World

This is a simple markdown document.

## Section 1

Some content here.

## Section 2

More content here.
"#;

    pub const MARKDOWN_WITH_CODE: &str = r#"# Code Example

Here is some Rust code:

```rust
fn main() {
    println!("Hello, world!");
}
```

And some Python:

```python
def hello():
    print("Hello, world!")
```
"#;

    pub const MARKDOWN_WITH_WIKILINKS: &str = r#"# Document with Wikilinks

Check out [[OtherPage]] for more info.

Also see [[SpecificPage|Custom Label]] for details.

Multiple links: [[A]], [[B]], [[C|Custom C]]
"#;

    pub const MARKDOWN_WITH_MATH: &str = r#"# Math Document

Inline math: $E = mc^2$

Block math:

$$
\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}
$$

Another equation: $\sum_{i=1}^{n} i = \frac{n(n+1)}{2}$
"#;

    pub const MARKDOWN_WITH_TABLE: &str = r#"# Table Example

| Header 1 | Header 2 | Header 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Cell 4   | Cell 5   | Cell 6   |
"#;

    pub const MARKDOWN_WITH_TASK_LIST: &str = r#"# Task List

- [x] Task 1 (completed)
- [ ] Task 2 (pending)
- [x] Task 3 (completed)
- [ ] Task 4 (pending)
"#;

    pub const MARKDOWN_WITH_FRONTMATTER: &str = r#"---
title: Test Document
author: Test Author
date: 2024-01-01
tags:
  - test
  - markdown
---

# Document Content

This document has YAML frontmatter.
"#;

    pub const MARKDOWN_COMPLEX: &str = r#"---
title: Complex Document
---

# Complex Markdown Document

This document tests various markdown features.

## Text Formatting

This is **bold**, this is *italic*, and this is ~~strikethrough~~.

## Links

- Regular link: [Example](https://example.com)
- Wikilink: [[InternalPage]]
- Wikilink with label: [[InternalPage|Custom Label]]
- Autolink: https://auto.example.com

## Code

Inline `code` and block:

```rust
fn example() -> i32 {
    42
}
```

## Lists

### Unordered
- Item 1
- Item 2
  - Nested item
- Item 3

### Ordered
1. First
2. Second
3. Third

### Task List
- [x] Done
- [ ] Todo

## Table

| Feature | Status |
|---------|--------|
| Tables  | Yes    |
| Links   | Yes    |

## Blockquote

> This is a blockquote.
> It can span multiple lines.

## Horizontal Rule

---

## Footnotes

Here is a footnote reference[^1].

[^1]: This is the footnote content.
"#;

    pub const EMPTY_MARKDOWN: &str = "";

    pub const WHITESPACE_ONLY_MARKDOWN: &str = "   \n\n\t\t\n   ";
}
