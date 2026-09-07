{
  description = "A markup language for creating plain-text diagrams";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/*";
    flake-parts.url = "github:hercules-ci/flake-parts";
    git-hooks.url = "github:cachix/git-hooks.nix";

    rust-flake.url = "github:juspay/rust-flake";
    rust-flake.inputs.nixpkgs.follows = "nixpkgs";

    crane.url = "github:ipetkov/crane";
  };

  outputs = inputs:
    let
      # Toolchain and crane lib are constructed OUTSIDE the perSystem module
      # config: crane.mkLib reads pkgs.stdenv eagerly, which recurses with
      # rust-flake's nixpkgs module (nixpkgs.hostPlatform is set from config).
      forEachSystem = inputs.nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      toolchainFor = system:
        let
          overlays = [ (import inputs.rust-flake.inputs.rust-overlay) ];
          pkgs = import inputs.nixpkgs { inherit system overlays; };
          toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          # Compose rust-flake's overrideToolchain with our clangStdenv
          # override: RUSTFLAGS below hardcode linker=clang + mold.
          craneLib =
            ((inputs.crane.mkLib pkgs).overrideScope (final: prev: {
              stdenvSelector = p: p.clangStdenv;
            })).overrideToolchain
              toolchain;
          # crt-static only makes sense for musl; mold only links ELF.
          cargoRustFlags = [
            "-C target-feature=+crt-static"
            "-C linker=clang"
            "-C link-arg=-fuse-ld=${pkgs.mold}/bin/mold"
          ];
          # Musl cross C toolchain for *-sys crates (libmimalloc-sys): the musl
          # Rust target ships no C compiler, and falling back to the glibc
          # gcc/clang compiles C objects against glibc headers (glibc >=2.41
          # redirects strtol to __isoc23_strtol in C23 mode), which musl cannot
          # link. Only meaningful on Linux; null elsewhere (Darwin has no musl
          # target).
          arch =
            if inputs.nixpkgs.lib.hasPrefix "aarch64-" system
            then "aarch64"
            else "x86_64";
          muslCc =
            if inputs.nixpkgs.lib.hasSuffix "-linux" system
            then
              (
                if arch == "x86_64"
                then pkgs.pkgsCross.musl64.stdenv.cc
                else pkgs.pkgsCross.aarch64-multiplatform-musl.stdenv.cc
              )
            else null;
          # Frontend dependencies, prefetched as a fixed-output derivation so
          # the sandboxed build can populate node_modules offline. When
          # glypho-web/package-lock.json changes, update this hash (set it to
          # lib.fakeHash and rebuild to get the correct value).
          npmDeps = pkgs.fetchNpmDeps {
            src = ./glypho-web;
            hash = "sha256-6jdKqsWod7igG22sjAOjkNBCnhuwDnQqWrd+2DwAEow=";
          };
        in
        { inherit toolchain craneLib cargoRustFlags npmDeps pkgs muslCc; };
      perSys = forEachSystem toolchainFor;
    in
    inputs.flake-parts.lib.mkFlake
      {
        inherit inputs;
        specialArgs = { inherit perSys; };
      }
      {
        systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ];

        imports = [
          inputs.rust-flake.flakeModules.default
          inputs.rust-flake.flakeModules.nixpkgs
          inputs.git-hooks.flakeModule
        ];

        perSystem =
          { config
          , lib
          , system
          , ...
          }:
          let
            # Derive platform facts from `system`, not pkgs.stdenv: reading
            # pkgs inside perSystem config recurses with rust-flake's
            # nixpkgs module (hostPlatform is set from config).
            isLinux = lib.hasSuffix "-linux" system;

            arch =
              if lib.hasPrefix "aarch64-" system
              then "aarch64"
              else "x86_64";

            # Static musl is the Linux default; Darwin builds use the native
            # apple-darwin target (musl does not exist for macOS).
            cargoTarget =
              if isLinux
              then "${arch}-unknown-linux-musl"
              else "${arch}-apple-darwin";

            inherit (perSys.${system}) cargoRustFlags npmDeps muslCc;

            # cc-crate env contract for the musl target, shared by crane args and
            # the devShell so both surfaces use the same C toolchain.
            muslCcEnv =
              if arch == "x86_64"
              then {
                CC_x86_64_unknown_linux_musl = "${muslCc}/bin/x86_64-unknown-linux-musl-cc";
                AR_x86_64_unknown_linux_musl = "${muslCc}/bin/x86_64-unknown-linux-musl-ar";
              }
              else {
                CC_aarch64_unknown_linux_musl = "${muslCc}/bin/aarch64-unknown-linux-musl-cc";
                AR_aarch64_unknown_linux_musl = "${muslCc}/bin/aarch64-unknown-linux-musl-ar";
              };

            # Args every build that compiles glypho itself (package, clippy,
            # tests) needs: build.rs shells out to npm for the embedded
            # frontend. Must NOT be in the shared crane.args, because
            # buildDepsOnly has a cleaned dummy src without glypho-web (the
            # npmConfigHook fails there).
            npmArgs = {
              inherit npmDeps;
              npmRoot = "glypho-web";

              # The sandbox has no /usr/bin/env; rewrite the shebangs of the
              # npm-cli shims (installed by npmConfigHook) so build.rs can run
              # `npm run build`.
              preBuild = ''
                patchShebangs glypho-web/node_modules
              '';
            };
            npmInputs = [ pkgs.nodejs pkgs.npmHooks.npmConfigHook ];

            # The crate's deps-only artifacts; rust-flake does not expose
            # craneBuild.cargoArtifacts, so rebuild the same derivation here
            # (identical args -> identical store path, fully shared).
            cargoArtifacts = perSys.${system}.craneLib.buildDepsOnly (
              config.rust-project.crates.glypho.crane.args
              // {
                inherit (config.rust-project) src;
                pname = "glypho";
                inherit (cargoPackage) version;
                cargoExtraArgs = "-p glypho";
                strictDeps = true;
              }
            );

            # Full args for any build that compiles glypho itself (clippy,
            # tests): raw crane.args plus what crate.nix would add (src, pname,
            # version, cargoExtraArgs) plus the npm plumbing build.rs needs.
            glyphoCompileArgs =
              config.rust-project.crates.glypho.crane.args
              // npmArgs
              // {
                inherit (config.rust-project) src;
                inherit cargoArtifacts;
                pname = "glypho";
                inherit (cargoPackage) version;
                cargoExtraArgs = "-p glypho";
                nativeBuildInputs =
                  config.rust-project.crates.glypho.crane.args.nativeBuildInputs
                  ++ npmInputs;
              };
            # Use the overlay-carrying pkgs from the hoist for consistency with
            # the toolchain and crane lib.
            pkgs = perSys.${system}.pkgs;

            cargoPackage = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package;
          in
          {
            # rust-flake's nixpkgs module would re-import nixpkgs from config,
            # recursing with pkgs.stdenv reads in perSystem config (crane.mkLib,
            # crate.nix defaults). Inject the hoisted overlay-carrying pkgs
            # through the sanctioned nixpkgs.pkgs option instead.
            nixpkgs.pkgs = pkgs;

            # Explicit formatter: without it `nix flake check`/`show` fail on
            # flake-parts' formatter heuristic.
            formatter = pkgs.nixpkgs-fmt;

            rust-project = {
              # Compose rust-flake's overrideToolchain with our clangStdenv
              # override: RUSTFLAGS below hardcode linker=clang + mold.
              crane-lib = perSys.${system}.craneLib;
              toolchain = perSys.${system}.toolchain;

              # crane's common cargo sources plus what the build actually embeds:
              # *.html (include_str templates), *.snap (insta snapshots), and the
              # glypho-web frontend (minus its build outputs, which would dirty
              # the fixed-hash npmDeps and constantly invalidate cargoArtifacts).
              src = pkgs.lib.cleanSourceWith {
                name = "source";
                src = pkgs.lib.fileset.toSource {
                  root = ./.;
                  fileset = pkgs.lib.fileset.unions [
                    (config.rust-project.crane-lib.fileset.commonCargoSources ./.)
                    (pkgs.lib.fileset.fileFilter (file: file.hasExt "html") ./.)
                    (pkgs.lib.fileset.fileFilter (file: file.hasExt "snap") ./.)
                    (pkgs.lib.fileset.fileFilter
                      (file:
                        file.type
                        == "directory"
                        || !(
                          lib.hasInfix "/node_modules/" file.name
                          || lib.hasInfix "/dist/" file.name
                          || lib.hasInfix "/.parcel-cache/" file.name
                        ))
                      ./glypho-web)
                  ];
                };
              };

              # The frontend build, static-musl linking, and openssl are project
              # concerns that crane passes through to the derivation unchanged.
              # crate.nix's own defaults for buildInputs/nativeBuildInputs read
              # the `pkgs` module argument (which recurses with rust-flake's
              # nixpkgs module), so we must define BOTH here — mkForce replaces
              # the default instead of merging into it.
              defaults.perCrate.crane.args = {
                CARGO_BUILD_TARGET = cargoTarget;

                # Target-scoped rustflags: CARGO_BUILD_RUSTFLAGS would also hit
                # host (gnu) crates — proc-macros and build scripts — where
                # +crt-static disables dylib output and breaks compilation.
                CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS = lib.optionalString (isLinux && arch == "x86_64") (lib.concatStringsSep " " cargoRustFlags);
                CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS = lib.optionalString (isLinux && arch == "aarch64") (lib.concatStringsSep " " cargoRustFlags);

                buildInputs = lib.optionals isLinux [ pkgs.openssl ];

                PKG_CONFIG_PATH = lib.optionalString isLinux "${pkgs.openssl.dev}/lib/pkgconfig";
                OPENSSL_INCLUDE_DIR = lib.optionalString isLinux "${pkgs.openssl.dev}/include";
                OPENSSL_LIB_DIR = lib.optionalString isLinux "${lib.getLib pkgs.openssl}/lib";

                nativeBuildInputs = lib.mkForce (with pkgs;
                  [
                    pkg-config
                    clang
                  ]
                  ++ lib.optionals isLinux [ mold llvmPackages.bintools muslCc ]);
              }
              # Musl C toolchain for *-sys crates (libmimalloc-sys); see the
              # hoist for why the glibc compiler cannot be used here.
              // lib.optionalAttrs isLinux muslCcEnv;

              crates.glypho = {
                path = ./.;

                # extraBuildArgs applies ONLY to the final package build
                # (not buildDepsOnly, whose cleaned dummy src lacks
                # glypho-web). It is merged with `//`, so concat
                # nativeBuildInputs explicitly.
                crane.extraBuildArgs =
                  npmArgs
                  // {
                    doCheck = false;

                    nativeBuildInputs =
                      config.rust-project.crates.glypho.crane.args.nativeBuildInputs
                      ++ npmInputs;

                    # UPX handles ELF reliably; Mach-O compression is unsupported.
                    postInstall = lib.optionalString isLinux ''
                      ${pkgs.upx}/bin/upx $out/bin/glypho
                    '';
                  };

                # The clippy check compiles glypho (running build.rs), so
                # it needs the same npm plumbing as the package build.
                crane.outputs.drv.clippy = config.rust-project.crane-lib.cargoClippy (
                  glyphoCompileArgs
                  // {
                    cargoClippyExtraArgs = "--all-targets --all-features -- --deny warnings";
                    meta.description = "Clippy check for the glypho crate";
                  }
                );

                # cargoDoc also runs build.rs (npm) — same treatment.
                crane.outputs.drv.doc = config.rust-project.crane-lib.cargoDoc (
                  glyphoCompileArgs
                  // {
                    RUSTDOCFLAGS = "-D warnings";
                    meta.description = "Rust docs for the glypho crate";
                  }
                );
              };
            };

            pre-commit.settings.hooks = {
              nixpkgs-fmt.enable = true;
              rustfmt.enable = true;
            };

            packages =
              {
                default = config.packages.glypho;

                # Not wired as a flake check: the native cargo-test CI matrix
                # (ubuntu/macos/windows) covers tests, and cargoTest uses a
                # separate cargoArtifacts build that doubles check time.
                runCargoTests = config.rust-project.crane-lib.cargoTest glyphoCompileArgs;
              }
              // lib.optionalAttrs isLinux {
                build_deb = pkgs.stdenv.mkDerivation {
                  name = "${cargoPackage.name}-${cargoPackage.version}-deb";
                  src = ./.;

                  nativeBuildInputs = [ pkgs.dpkg ];
                  buildInputs = [ config.packages.glypho ];

                  buildPhase = ''
                    mkdir -p package/usr/bin
                    mkdir -p package/DEBIAN

                    cp ${config.packages.glypho}/bin/glypho package/usr/bin/

                    # Create control file
                    cat > package/DEBIAN/control <<EOF
                    Package: ${cargoPackage.name}
                    Version: ${cargoPackage.version}
                    Section: utils
                    Priority: optional
                    Architecture: amd64
                    Maintainer: Roberto Galaz <r.galaz11@gmail.com>
                    Description: ${cargoPackage.description}
                     ${cargoPackage.description}
                    EOF
                  '';

                  installPhase = ''
                    mkdir -p $out
                    dpkg-deb --build package $out/${cargoPackage.name}_${cargoPackage.version}_amd64.deb
                  '';
                };
              };

            devShells.default = pkgs.mkShell (
              {
                inputsFrom = [ config.devShells.rust config.pre-commit.devShell ];

                CARGO_BUILD_TARGET = cargoTarget;

                packages = with pkgs;
                  [
                    sccache
                    clang
                    git-cliff
                    gh
                    coreutils
                    rust-analyzer
                    skopeo
                    watchexec
                    systemfd
                    bacon
                    cargo-audit
                    cargo-machete
                    cargo-nextest
                    cargo-insta
                  ]
                  ++ lib.optionals isLinux [
                    dpkg
                    upx
                    openssl
                    mold
                    llvmPackages.bintools
                    gdb
                    muslCc
                  ];
              }
              // lib.optionalAttrs isLinux ({
                PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
                OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
                OPENSSL_LIB_DIR = "${lib.getLib pkgs.openssl}/lib";
              }
              # Musl C toolchain for *-sys crates; same contract as crane args.
              // muslCcEnv
              # Target-scoped rustflags: unscoped CARGO_BUILD_RUSTFLAGS would
              # also hit host (gnu) proc-macros and build scripts, where
              # +crt-static disables dylib output and breaks compilation.
              // (
                if arch == "x86_64"
                then {
                  CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS = lib.concatStringsSep " " cargoRustFlags;
                }
                else {
                  CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS = lib.concatStringsSep " " cargoRustFlags;
                }
              ))
            );
          };
      };
}
