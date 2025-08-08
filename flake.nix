{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
    git-commit-hooks.url = "github:cachix/git-hooks.nix";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
    git-commit-hooks,
    self,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [
          (import rust-overlay)
        ];

        stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.stdenv;
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = ["x86_64-unknown-linux-musl"];
        };

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
          inherit stdenv;
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        pkgs = import nixpkgs {inherit system overlays;};
        rustVersion = rustToolchain;

        src = let
          fs = pkgs.lib.fileset;
          unfilteredRoot = ./.; # The original, unfiltered source
          files = pkgs.lib.fileset.unions [
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            (pkgs.lib.fileset.fileFilter (file: file.hasExt "html") unfilteredRoot)
            ./glypho-web
          ];

          source = pkgs.lib.fileset.toSource {
            root = unfilteredRoot;
            fileset = files;
          };
        in
          pkgs.lib.cleanSourceWith {
            src = source;
            name = "source";
          };

        commonArgs = {
          inherit src;
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static -C linker=clang -C link-arg=-fuse-ld=${pkgs.mold}/bin/mold";
          buildInputs = with pkgs; [
            openssl
            nodejs
            nodePackages.pnpm
          ];
          nativeBuildInputs = with pkgs; [
            clang
            mold
            upx
            pkg-config
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        glypho = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            pname = "glypho";
            postInstall = ''
              ${pkgs.upx}/bin/upx $out/bin/${glypho.pname}
            '';
          }
        );

        runCargoTests = craneLib.cargoTest (commonArgs // {inherit src cargoArtifacts;});
      in {
        packages = {
          inherit glypho runCargoTests;
          default = glypho;
        };

        checks = {
          inherit runCargoTests;
          pre-commit-check = git-commit-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              nixpkgs-fmt.enable = true;
              rustfmt.enable = true;
              # some hooks provide settings
            };
          };
        };

        devShells.default = (
          let
            moldDevShell = craneLib.devShell.override {
              # For example, use the mold linker
              mkShell = pkgs.mkShell.override {
                inherit stdenv;
              };
            };
          in
            moldDevShell {
              RUSTC_WRAPPER = "${pkgs.sccache}/bin/sccache";
              CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
              CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static -C linker=clang -C link-arg=-fuse-ld=${pkgs.mold}/bin/mold";
              inherit (self.checks.${system}.pre-commit-check) shellHook;

              nativeBuildInputs = with pkgs; [
                sccache
                pkg-config
                clang
                mold
              ];

              buildInputs = [
                (rustVersion.override {
                  extensions = ["rust-src" "rust-analyzer" "rustc" "cargo" "clippy"];
                })
                self.checks.${system}.pre-commit-check.enabledPackages
              ];

              packages = with pkgs; [
                clang
                git-cliff
                nodejs
                nodePackages.pnpm
                mold
                upx
                coreutils
                rust-analyzer
                skopeo
                watchexec
                systemfd
                bacon
                openssl
              ];
            }
        );
      }
    );
}
