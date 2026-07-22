## Context

Glypho is a single-binary Rust CLI: an axum server renders Markdown to an embedded HTML template and live-reloads via datastar SSE. Today it is Linux-only in practice:

```
   COUPLING POINT                TODAY                         BLOCKS
   ═══════════════════════════════════════════════════════════════════
   build.rs frontend step       `Command::new("npm")` +         Windows (no cp,
                                  `Command::new("cp")`            npm.cmd shim)
   instance liveness            `std::fs::exists("/proc/pid")`   macOS, Windows
   runtime dir                  `xdg` crate, returns None        Windows
                                  on non-Linux
   flake.nix                    musl target + mold linker        macOS (mold is
                                  hardcoded for all systems       ELF-only)
   tests                        absolute Unix paths              Windows
   CI                           flakehub-publish only            all testing/release
```

Constraints:

- The flake uses `flake-utils.lib.eachDefaultSystem`, which already enumerates `x86_64-linux`, `aarch64-linux`, `x86_64-darwin`, `aarch64-darwin` — but the build args assume Linux everywhere, so Darwin evaluation fails today.
- Nix cannot target Windows (no mingw cross in the default set; MSVC impossible in pure Nix). Windows artifacts must come from a native GitHub runner using cargo.
- `reqwest` is used only to POST to `localhost` — plain HTTP, no TLS needed, so the current `default-features = false` (no TLS) works on all platforms unchanged.
- `async-watcher`/`notify`, `tokio`, `axum`, `open` are already cross-platform.

## Goals / Non-Goals

**Goals:**
- `cargo build` / `cargo test` succeed on Linux, macOS, Windows with identical feature behavior.
- `nix build .#glypho` succeeds on all four `eachDefaultSystem` systems; musl-static stays the Linux default.
- Tag-triggered GitHub release with per-OS artifacts: musl binary + `.deb` (Linux), Mach-O binary (macOS), `.exe` (Windows).
- No Linux-only APIs outside `cfg(target_os = "linux")` gates.

**Non-Goals:**
- Windows via Nix or cross-compilation — rejected (see Decisions).
- Code-signing / notarization / installers.
- Flatpak changes.
- Changing rendering/SSE/watch behavior.

## Decisions

### D1 — build.rs: pure-Rust copy, `npm` via `which` lookup

**Decision:** Replace both `Command` shell-outs. Copy `glypho-web/dist/index.html` to `src/template.html` with `std::fs::copy`. Detect the package manager with the `which` crate (resolves `npm.cmd`/`npm.exe` on Windows), then run it with `Command`.

**Alternatives considered:**
- *Prebuild and commit `template.html`*: removes the npm dependency from `cargo build` entirely and makes `cargo install` from crates.io work without Node. Attractive, but the file is 1.5 MB generated output; committing it invites drift between `glypho-web/src` and the committed artifact. Rejected for now — keep build-time generation, revisit if crates.io installs without Node become a requirement.
- *Keep `cp` behind cfg*: leaves the Windows build doing something different from Linux. Rejected — one code path.

**Note:** `build.rs` currently ignores the exit status of both commands (`.status()?` without checking `success()`). While touching this, assert success so a failed frontend build fails the cargo build on all platforms.

### D2 — Instance liveness: probe the port, not `/proc`

**Decision:** Replace `std::fs::exists("/proc/{pid}")` with a two-step check:

1. GET `http://localhost:{port}/` with a short timeout — success means a live glypho (or at least a live server) owns the port; POST `/add` and exit as today.
2. If unreachable, treat the pidfile as stale: delete it and continue startup.

**Alternatives considered:**
- *cfg-gated per-OS liveness* (`/proc` on Linux, `libproc`/`sysinfo` on macOS, `OpenProcess` on Windows): preserves exact pid semantics but pulls in per-OS crates and three code paths to test. Rejected.
- *sysinfo crate for all platforms*: adds a dependency to re-implement what a TCP probe already proves — the thing we actually care about is "is the server answering", not "does pid N exist" (pids get reused; a reused pid would be a false positive under the current scheme anyway).

**Trade-off accepted:** if a *different* process happens to own the recorded port, we'll POST `/add` to it. That was already possible today (pid reuse); the pidfile records the port we bound, so collisions require a stale pidfile plus a squatter — acceptable.

### D3 — Runtime directory: `etcetera` crate

**Decision:** Replace `xdg` with `etcetera`'s `choose_app_strategy(AppStrategyArgs { top_level_domain: "dev", author: "trafkin", app_name: "glypho" })`. Fallback chain: `runtime_dir()` (XDG strategy on Linux: `$XDG_RUNTIME_DIR/glypho`, same as today) → `state_dir()` → `data_dir()` (non-Option on all strategies, so the chain always resolves). On macOS the default strategy is also XDG (CLI-tool convention); on Windows it's the Windows Known-Folder strategy, where `runtime_dir()`/`state_dir()` return `None` and the pidfile lands in the data dir.

**Why etcetera over `directories`:** `directories` was archived on GitHub in Feb 2025 — development did move to Codeberg and is alive, but `etcetera` is the more active project (latest release 2025-10, commits 2025-11, 84M downloads vs 61M), GitHub-native, and offers the same runtime/state/data API with per-OS strategy choice. The API contact surface is ~4 functions, so future migration cost is low either way.

**Behavior preservation:** on Linux, `etcetera`'s XDG app strategy joins the unixy app name (`"glypho"`) under `$XDG_RUNTIME_DIR` — byte-identical to the old `xdg::BaseDirectories::with_prefix("glypho")` path.

**Alternatives considered:**
- *keep `xdg` + hand-rolled Windows paths* — rejected; platform-dir logic is exactly what these crates exist for.
- *`directories` crate* — works and API-equivalent, but the archived-upstream signal is worse; etcetera wins on maintenance evidence.
- *`dirs` crate* — same maintainers as `directories`, but no per-project path composition or runtime_dir; too weak.

### D4 — Flake: per-system target/linker selection

**Decision:** Compute target triple and linker settings from `system` inside the existing `eachDefaultSystem` lambda:

```
   system            CARGO_BUILD_TARGET           linker
   ═══════════════════════════════════════════════════════════════
   *-linux           <arch>-unknown-linux-musl    clang + mold, +crt-static
   *-darwin          <arch>-apple-darwin          default (ld64), no mold,
                                                   no crt-static, no upx
```

- mold, musl flags, and `upx` move behind `pkgs.stdenv.isLinux` / `isDarwin` conditionals in both `commonArgs` and the dev shell (upx on Mach-O is unreliable; skip postInstall compression on Darwin).
- The dev shell keeps working on macOS — this is the *only* way to develop on the project from a Mac without Homebrew sprawl.
- `deb-package` stays defined but is only meaningful on `x86_64-linux`; guard with `lib.optionals stdenv.isLinux`.

**Alternative considered:** *crane cross-compilation from one Linux builder to all targets* — attractive for CI speed, but cross to Darwin from Linux requires osxcross (unfree SDK, not viable in nixpkgs), and cross to Windows produces mingw binaries we can't test natively. Rejected: build each OS on its native runner.

### D5 — Windows from a native runner, not Nix

**Decision:** The release matrix uses `windows-latest` with plain `cargo build --release` (via `dtolnay/rust-toolchain`, reading `rust-toolchain.toml`). Nix orchestrates Linux + macOS only.

**Rationale:** see Constraints. Forcing Windows through Nix buys nothing: the output couldn't be signed/tested in Nix anyway, and the runner image has everything needed.

### D6 — CI topology: two workflows

**Decision:**

- `ci.yml` (push/PR): test matrix — `ubuntu-latest`, `macos-latest`, `windows-latest` run `cargo test` (+ `nix flake check` on the two Nix systems). Node setup step runs the frontend build first, mirroring `build.rs` needs.
- `release.yml` (tag `v*`): builds artifacts per D4/D5, then a final job assembles the GitHub release via `gh release create` with all artifacts (musl tarball, `.deb`, macOS tarball, `glypho.exe` zip). Replaces the manual `gh release` steps in `docs/deb-release.md`.

**Alternative considered:** one workflow doing both — rejected; test-on-push and release-on-tag have different triggers and failure semantics.

### D7 — Test path portability

**Decision:** Replace hardcoded Unix path literals in tests (`/tmp/test.md`, `/path/to/file.md`, `/nonexistent/path/...`) with `tempfile::TempDir`-based paths or relative-to-tempdir constructions. Tests asserting "file not found" use a path generated by joining a nonexistent name onto a `TempDir` (portable "definitely missing" location).

## Risks / Trade-offs

- **Port-squatting false positive (D2):** a stale pidfile + unrelated server on the recorded port makes glypho POST `/add` to a stranger. Mitigation: keep the `/add` payload harmless (path only); the existing code has the same class of bug via pid reuse, so this is not a regression.
- **npm detection on Windows (D1):** `which` handles `.cmd` resolution, but corporate Windows environments with npm only via `nvm` shims can still fail. Mitigation: clear build error message naming the missing tool; D1's failure assertion makes this loud instead of silent.
- **Darwin flake eval (D4):** per-system conditionals touch most of `commonArgs`; a bad conditional breaks Linux (the working platform). Mitigation: `nix flake check` on Linux in CI gates every PR; verify `nix eval .#packages.x86_64-darwin.glypho` from Linux CI even though the *build* needs a Darwin runner.
- **Release workflow is untestable until a tag:** first real tag may need fixes. Mitigation: support `workflow_dispatch` trigger so the pipeline can be rehearsed without publishing.
- **`etcetera` vs `xdg` fallback paths (D3):** on Linux-without-`$XDG_RUNTIME_DIR`, the pidfile lands in a different directory than before (`~/.local/state/glypho` vs `~/.local/share/glypho`). Only affects stale-pidfile cleanup; acceptable.
