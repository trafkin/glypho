## 1. Build Script Portability

- [x] 1.1 Add `which` to `[build-dependencies]` in Cargo.toml
- [x] 1.2 Rewrite build.rs: locate npm via `which` (handles npm.cmd on Windows), run `npm run build` in glypho-web, and fail the build on non-zero exit status
- [x] 1.3 Replace the `cp` Command with `std::fs::copy` from `glypho-web/dist/index.html` to `src/template.html`; error if the source is missing
- [x] 1.4 Verify `cargo build` succeeds and template.html is regenerated (Linux smoke test)

## 2. Process Lifecycle Portability

- [x] 2.1 Swap `xdg` for `etcetera` in Cargo.toml; update `write_runtime`, `cleanup`, and `check_uniqueness` in src/main.rs to use `choose_app_strategy(AppStrategyArgs { top_level_domain: "dev", author: "trafkin", app_name: "glypho" })` with runtime_dir→state_dir→data_dir fallback
- [x] 2.2 Replace the `/proc/{pid}` liveness check in `check_uniqueness` with an HTTP probe of `http://localhost:{port}/` (short timeout); on success POST /add and exit, on failure delete stale pidfile and continue startup
- [x] 2.3 Add unit tests: stale-pidfile path proceeds to startup; pidfile round-trip (write_runtime → read back) works under a temp HOME/XDG dir
- [x] 2.4 Remove the `xdg` dependency and confirm no remaining imports

## 3. Test Path Portability

- [x] 3.1 Replace hardcoded Unix path literals in src/state.rs tests (`/tmp/test.md`, `/path/to/file.md`) with `tempfile::TempDir`-based paths
- [x] 3.2 Rework the "file not found" render test to use a nonexistent path joined under a TempDir instead of `/nonexistent/path/file.md`
- [x] 3.3 Audit src/cli.rs and tests/ for other Unix-only path assumptions and fix them
- [x] 3.4 Verify `cargo check --target x86_64-pc-windows-msvc` and `--target aarch64-apple-darwin` compile (targets added via rustup)

## 4. Nix Multi-System Flake

- [x] 4.1 Parameterize `CARGO_BUILD_TARGET` and linker flags by system in flake.nix: musl + mold + crt-static for `*-linux`, `apple-darwin` default linker for `*-darwin`
- [x] 4.2 Gate mold, musl flags, and the UPX `postInstall` behind `stdenv.isLinux` conditionals in commonArgs and the package definition
- [x] 4.3 Gate `deb-package` and dpkg inputs to Linux-only; keep `nix build .#build_deb` working on x86_64-linux
- [x] 4.4 Fix dev shell inputs so `nix develop` evaluates on Darwin (remove/conditionalize Linux-only build inputs)
- [x] 4.5 Verify `nix build .#glypho` and `nix flake check` still pass on x86_64-linux; `nix eval` the Darwin package paths to confirm evaluation succeeds

## 5. CI Workflow

- [x] 5.1 Create `.github/workflows/ci.yml`: matrix over ubuntu-latest, macos-latest, windows-latest; checkout, Node setup, `cargo test` with the repo's rust-toolchain
- [x] 5.2 Add `nix flake check` (via DeterminateSystems/determinate-nix-action) to the Linux and macOS CI jobs
- [ ] 5.3 Verify the workflow passes on a PR (all three OSes green)

## 6. Release Workflow

- [x] 6.1 Create `.github/workflows/release.yml` triggered on `v*` tags and `workflow_dispatch`
- [x] 6.2 Linux job: `nix build .#glypho` → tarball; `nix build .#build_deb` → .deb artifact upload
- [x] 6.3 macOS job: `nix build .#glypho` → tarball artifact upload
- [x] 6.4 Windows job: cargo build --release → zip glypho.exe artifact upload
- [x] 6.5 Assembly job: download all artifacts and `gh release create` for the tag (skipped on workflow_dispatch rehearsal)
- [ ] 6.6 Rehearse via workflow_dispatch, then cut a real tag to validate end-to-end

## 7. Docs and Cleanup

- [x] 7.1 Update README installation section with macOS/Windows instructions (cargo install, release binaries)
- [x] 7.2 Update docs/deb-release.md and docs/cargo-release.md to reference the automated release workflow instead of manual artifact upload
- [x] 7.3 Note in CHANGELOG.md that cross-platform support and CI/release automation were added
