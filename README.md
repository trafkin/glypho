# Glypho 

<img alt="Glypho Logo" src="./assets/glypho-logo-transparent.png" align="right" width="120">

**Glypho** is a fast, minimal CLI tool written in Rust to render Markdown files in a clean, pretty format via a local web server.

- View your Markdown files in a beautiful format
- Fast startup and live preview through a local web server

## Why Glypho?

Glypho was created to make reading Markdown as frictionless as possible — especially when you just want to **open a file and read**. No need to open a full editor or configure plugins. Just run the command and go.

> no heavy dependencies — just Markdown and Rust.

## Installation

Glypho runs on Linux, macOS, and Windows.

### Prebuilt binaries

Download the archive for your platform from the [GitHub releases](https://github.com/trafkin/glypho/releases) page:

- Linux: `glypho-x86_64-unknown-linux-musl.tar.gz` (static, runs anywhere) or the `.deb` package
- macOS: `glypho-aarch64-apple-darwin.tar.gz`
- Windows: `glypho-x86_64-pc-windows-msvc.zip`

On Debian/Ubuntu you can install the `.deb` directly:

```sh
sudo apt install ./glypho_<version>_amd64.deb
```

### From crates.io

Requires [Node.js](https://nodejs.org/) (npm on PATH) — the build script bundles the frontend.

```sh
cargo install glypho
```

### From source

1. Pull the repository and go to the root directory
2. Run `cargo install --path .`

### With Nix (Linux and macOS)

```sh
nix run github:trafkin/glypho -- <file.md>
```

Or enter the dev shell for hacking on the project:

```sh
nix develop
```

## Usage

```sh
glypho README.md          # open a Markdown file in the preview
glypho --port 3000 README.md   # on a fixed port
glypho --no-browser note.md    # don't open the browser
glypho --server-mode           # start empty, no initial file
```

![Demo Animation](./assets/glypho.gif)

### Server mode

By default Glypho needs a file to open. `--server-mode` starts the preview
without one, showing an **open-form** page where you can type the path of a
Markdown file to open. Relative paths are resolved against the directory you
started Glypho in.

You can also hand files to a running (server-mode or normal) instance from the
command line — a second `glypho other.md` invocation opens the file in the
existing preview instead of starting a second server.

## MCP server (for AI agents)

Glypho exposes an [MCP](https://modelcontextprotocol.io) (Model Context
Protocol) server over Streamable HTTP on the same port as the preview, so AI
agents can detect Markdown files they created and open them in the live
preview.

Start Glypho with a fixed port:

```sh
glypho --port 3000 README.md
```

Then point your MCP client at `http://localhost:3000/mcp`. For example, in a
client that accepts a URL-based MCP server configuration:

```json
{
  "mcpServers": {
    "glypho": {
      "url": "http://localhost:3000/mcp"
    }
  }
}
```

The server exposes three tools:

| Tool | Description |
| --- | --- |
| `detect_markdown_files` | Scan a directory (or the client's first MCP root, or the current working directory) for Markdown files and return a question listing them. |
| `open_markdown_files` | Track and open the chosen files in the live preview (`files` for explicit paths, `open_all: true` for every file proposed by the last scan). |
| `list_tracked_files` | Show which files Glypho is tracking and which one is active. |

The typical agent flow is: call `detect_markdown_files`, ask the user which
files to open, then call `open_markdown_files` with the selection — so large
batches of generated Markdown never flood the preview unprompted. The MCP
endpoint binds to `127.0.0.1` only, like the rest of the server.

Opening files over MCP keeps Glypho's normal behavior: the preview is for the
human. If a preview tab is already connected, it switches to the opened file
live; if no preview is connected (the tab was closed), Glypho opens your
default browser, exactly as it does at startup. Start with `--no-browser` to
suppress all browser opening. The tools themselves return only file paths and
status — they never send rendered content back to the agent.
