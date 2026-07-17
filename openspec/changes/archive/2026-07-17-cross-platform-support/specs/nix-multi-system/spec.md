## ADDED Requirements

### Requirement: Flake builds on all default systems
The Nix flake SHALL provide a working `glypho` package on every system enumerated by its `eachDefaultSystem` output: `x86_64-linux`, `aarch64-linux`, `x86_64-darwin`, and `aarch64-darwin`.

#### Scenario: Darwin build
- **WHEN** `nix build .#glypho` runs on an aarch64-darwin or x86_64-darwin machine
- **THEN** the build completes producing a native Mach-O binary, without attempting to use mold, musl, or crt-static flags

#### Scenario: Linux build unchanged
- **WHEN** `nix build .#glypho` runs on x86_64-linux
- **THEN** the build completes producing a statically-linked musl binary as before

### Requirement: Linker and target are system-appropriate
Target triple, linker, and packaging post-processing (e.g. UPX) SHALL be selected per system rather than hardcoded.

#### Scenario: Per-system evaluation
- **WHEN** the flake is evaluated for any supported system
- **THEN** `CARGO_BUILD_TARGET`, linker flags, and `postInstall` steps match that system's platform conventions (mold only on Linux ELF targets, UPX only where supported)

### Requirement: Dev shell works on macOS
The flake dev shell SHALL evaluate and enter successfully on Darwin systems with the same Rust toolchain and frontend tooling available as on Linux.

#### Scenario: Darwin dev shell
- **WHEN** a developer runs `nix develop` on macOS
- **THEN** they get cargo (per `rust-toolchain.toml`), nodejs/pnpm, and pre-commit hooks without Linux-only build inputs breaking evaluation

### Requirement: Debian package remains Linux-only
The `.deb` package output SHALL continue to be produced on x86_64-linux and SHALL NOT be required to exist on other systems.

#### Scenario: deb build on Linux
- **WHEN** `nix build .#build_deb` runs on x86_64-linux
- **THEN** a `.deb` containing the musl glypho binary is produced as before
