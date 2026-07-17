# ci-release Specification

## Purpose

GitHub Actions provide a test matrix across Linux, macOS, and Windows plus a tag-triggered release pipeline producing per-OS artifacts.

## Requirements

### Requirement: Test suite runs on all three OSes
CI SHALL run the full `cargo test` suite on Linux, macOS, and Windows runners on every push and pull request.

#### Scenario: Matrix pass
- **WHEN** a push or PR triggers the CI workflow
- **THEN** jobs run on `ubuntu-latest`, `macos-latest`, and `windows-latest`, each executing `cargo test` with the repository's pinned toolchain

#### Scenario: Nix checks gate Linux and macOS
- **WHEN** the CI workflow runs on Linux and macOS
- **THEN** `nix flake check` (or the flake's test package) also runs, catching per-system Nix regressions

### Requirement: Tests use portable paths
Unit and integration tests SHALL NOT depend on Unix-only absolute paths (e.g. `/tmp`, `/proc`, `/path/to`) so they pass unchanged on Windows.

#### Scenario: Windows test run
- **WHEN** `cargo test` executes on a Windows runner
- **THEN** all tests pass, with temporary files created via `tempfile` and "missing file" cases using nonexistent paths under a temp directory

### Requirement: Tag-triggered release with per-OS artifacts
Pushing a version tag SHALL build release artifacts for Linux (musl binary and `.deb`), macOS (native binary), and Windows (`.exe`), and publish them to a GitHub release.

#### Scenario: Release assembly
- **WHEN** a tag matching `v*` is pushed
- **THEN** the release workflow builds each artifact on its native runner and attaches all artifacts to one GitHub release named for the tag

#### Scenario: Rehearsal without publishing
- **WHEN** the release workflow is triggered manually via `workflow_dispatch`
- **THEN** artifacts are built and uploaded as workflow artifacts without creating a public release

### Requirement: Existing FlakeHub publish keeps working
The existing `flakehub-publish-rolling.yml` workflow SHALL remain functional and unchanged in behavior.

#### Scenario: Main-branch push
- **WHEN** a commit is pushed to `main`
- **THEN** the flake is published to FlakeHub exactly as before this change
