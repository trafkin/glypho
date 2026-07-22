## ADDED Requirements

### Requirement: Frontend asset build is platform-agnostic
The build script SHALL build the frontend bundle and place the generated HTML template using only Rust-standard mechanisms and PATH-resolvable tooling, so that `cargo build` succeeds on Linux, macOS, and Windows.

#### Scenario: Build on any supported OS
- **WHEN** a developer runs `cargo build` on Linux, macOS, or Windows with npm installed and on PATH
- **THEN** the frontend bundle is built and `src/template.html` is produced without invoking Unix-only utilities (no `cp`, no shell-specific commands)

#### Scenario: npm resolved portably
- **WHEN** the build script locates the npm executable
- **THEN** it resolves the executable via PATH lookup that handles platform executable extensions (e.g. `npm.cmd` on Windows)

### Requirement: Build failures are not silently ignored
The build script SHALL fail the cargo build when the frontend build or template copy step fails.

#### Scenario: Frontend build error
- **WHEN** `npm run build` exits with a non-zero status or npm is not found
- **THEN** `cargo build` fails with an error identifying the failing step

#### Scenario: Missing frontend output
- **WHEN** the build output file `glypho-web/dist/index.html` does not exist after the build step
- **THEN** `cargo build` fails instead of silently continuing

### Requirement: No Linux-only APIs outside cfg gates
Platform-specific APIs SHALL be confined to `cfg`-gated code paths so the crate compiles on all three target OSes without warnings about unreachable or missing APIs.

#### Scenario: Cross-compile check
- **WHEN** `cargo check` runs targeting `x86_64-pc-windows-msvc` and `aarch64-apple-darwin`
- **THEN** compilation succeeds with no unresolved imports or platform-gated symbol errors
