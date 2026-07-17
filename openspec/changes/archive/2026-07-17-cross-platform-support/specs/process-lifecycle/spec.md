## ADDED Requirements

### Requirement: Single-instance detection without /proc
The application SHALL detect an already-running instance using a mechanism that works on Linux, macOS, and Windows, and SHALL NOT read `/proc` outside `cfg(target_os = "linux")` code.

#### Scenario: Running instance detected via port probe
- **WHEN** a runtime pidfile exists and an HTTP server responds at the recorded port
- **THEN** the new process submits its file to the running instance via `POST /add` and exits with status 0

#### Scenario: Stale pidfile cleanup
- **WHEN** a runtime pidfile exists but no server responds at the recorded port
- **THEN** the stale pidfile is removed and startup continues as a fresh primary instance

### Requirement: Runtime files use platform-appropriate directories
Pidfile and runtime state SHALL be stored via a platform-aware project-directories resolution so paths are correct on every OS.

#### Scenario: Linux path unchanged
- **WHEN** glypho runs on Linux with `$XDG_RUNTIME_DIR` set
- **THEN** the pidfile is placed under `$XDG_RUNTIME_DIR/glypho/`, matching prior behavior

#### Scenario: Non-Linux fallback
- **WHEN** glypho runs on macOS or Windows where no runtime-dir concept exists
- **THEN** the pidfile is placed in the platform state/data directory for the application and is still found by subsequent invocations

### Requirement: Clean shutdown removes runtime artifacts
On Ctrl+C shutdown the application SHALL remove its pidfile regardless of platform.

#### Scenario: Shutdown cleanup
- **WHEN** the user interrupts the server with Ctrl+C on any supported OS
- **THEN** the pidfile is deleted and a later invocation starts a fresh primary instance
