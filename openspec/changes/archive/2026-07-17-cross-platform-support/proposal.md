## Why

Glypho is a Markdown preview CLI that today only builds and runs correctly on x86_64 Linux. Its build script shells out to Unix tools (`cp`, `npm`), its single-instance liveness check reads Linux-only `/proc`, its runtime directory uses the XDG spec with no Windows fallback, and its Nix flake hardcodes a musl-static target with an ELF-only linker (mold). There is also no automated CI — the only workflow publishes the flake to FlakeHub. As a result the tool cannot be built, tested, or released on macOS or Windows.

## What Changes

- **Portable frontend build (`build.rs`)**: replace the `cp` shell-out and bare `npm` invocation with pure-Rust file copying and cross-platform npm detection (handles `npm.cmd` on Windows). No behavioral change on Linux.
- **Cross-platform single-instance gate (`src/main.rs`)**: replace the `/proc/<pid>` liveness probe with a portable check — TCP probe of the recorded port, falling back to a `cfg`-gated OS check. Process liveness via `/proc` stays Linux-only behind `cfg(target_os = "linux")`; other platforms probe the HTTP endpoint.
- **Portable runtime/state directory**: migrate from the `xdg` crate (returns `None` on Windows) to the `etcetera` crate's app strategy, with platform-appropriate runtime/state paths. Keeps current Linux paths identical.
- **Multi-system Nix flake**: parameterize target triple and linker per system. musl-static + mold remains for Linux; Darwin drops mold and targets `apple-darwin`; no Windows Nix target (Nix cannot target Windows).
- **Test suite portability**: replace hardcoded Unix absolute paths in unit tests (`/tmp/...`, `/path/to/...`) with `tempfile`-based paths so tests pass on all three OSes.
- **CI/CD pipelines**: new GitHub Actions workflows — a test matrix (Linux, macOS, Windows) running `cargo test` and `nix flake check` where applicable, and a release workflow that builds per-OS artifacts (musl binary + `.deb` via Nix on Linux, Mach-O via Nix on macOS, `.exe` via cargo on Windows) and publishes a GitHub release on tag.

### Non-goals

- No Flatpak or additional Linux package formats beyond the existing `.deb`.
- No code-signing, notarization, or installers for macOS/Windows in this change.
- No functional changes to Markdown rendering, SSE, or watching behavior.

## Capabilities

### New Capabilities
- `cross-platform-build`: the build script and dependency/toolchain detection work identically on Linux, macOS, and Windows.
- `process-lifecycle`: single-instance detection, runtime pidfile management, and cleanup behave correctly per-OS.
- `nix-multi-system`: the flake exposes working `glypho` packages and checks on `x86_64-linux`, `aarch64-linux`, `x86_64-darwin`, `aarch64-darwin`.
- `ci-release`: GitHub Actions test matrix across the three OSes plus a tag-triggered release pipeline producing per-OS artifacts.

### Modified Capabilities
<!-- No existing specs in openspec/specs/, so no modified capabilities. -->

## Impact

- **Code**: `build.rs`, `src/main.rs` (cleanup/check_uniqueness/write_runtime), `Cargo.toml` (swap `xdg` for `directories`, possibly drop unused deps), `flake.nix` (per-system target/linker logic), unit tests in `src/state.rs`, `src/cli.rs`, `tests/`.
- **Dependencies**: remove `xdg`, add `etcetera`. Verify `reqwest` TLS features and `open` crate behavior on Windows/macOS. Possibly gate `mold`/musl-only build inputs in Nix.
- **CI/CD**: new files under `.github/workflows/` (test matrix, release). Existing `flakehub-publish-rolling.yml` unchanged.
- **Docs**: update README install instructions for macOS/Windows; new release docs for the CI pipeline.
- **Tooling**: Windows builds come from a native GitHub runner (cargo), not Nix — Nix is used only where it can target (Linux, macOS).
