{
  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/*";
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
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        pkgs = import nixpkgs {inherit system overlays;};
        rustVersion = rustToolchain;

        src = let
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
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
          OPENSSL_LIB_DIR = "${pkgs.lib.getLib pkgs.openssl}/lib";
          buildInputs = with pkgs; [
            rustToolchain
            mold
            openssl
            nodejs
            pnpm
          ];
          nativeBuildInputs = with pkgs; [
            rustToolchain
            clang
            mold
            upx
            pkg-config
            llvmPackages.bintools
            gdb
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        glypho = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            doCheck = false;
            pname = "glypho";
            postInstall = ''
              ${pkgs.upx}/bin/upx $out/bin/${glypho.pname}
            '';
          }
        );

        deb-package = pkgs.stdenv.mkDerivation {
          name = "glypho";
          src = ./.;

          nativeBuildInputs = [pkgs.dpkg];
          buildInputs = [glypho];

          buildPhase = ''
            mkdir -p package/usr/bin
            mkdir -p package/DEBIAN

            cp ${glypho}/bin/glypho package/usr/bin/

            # Create control file
            cat > package/DEBIAN/control <<EOF
            Package: glypho
            Version: 0.1.0
            Section: utils
            Priority: optional
            Architecture: amd64
            Maintainer: Your Name <your.email@example.com>
            Description: My Rust Application
             A simple Rust application packaged as .deb
            EOF
          '';

          installPhase = ''
            mkdir -p $out
            dpkg-deb --build package $out/glypho_0.1.0_amd64.deb
          '';
        };

        runCargoTests = craneLib.cargoTest (commonArgs // {inherit src cargoArtifacts;});
      in {
        packages = {
          inherit glypho runCargoTests;
          default = glypho;
          build_deb = deb-package;
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
          craneLib.devShell {
            inherit (self.checks.${system}.pre-commit-check) shellHook stdenv;
            CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
            OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
            OPENSSL_LIB_DIR = "${pkgs.lib.getLib pkgs.openssl}/lib";
            CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static -C linker=clang -C link-arg=-fuse-ld=${pkgs.mold}/bin/mold";

            nativeBuildInputs = with pkgs; [
              rustToolchain
              sccache
              mold
              pkg-config
              openssl
              clang
              mold
              llvmPackages.bintools
            ];

            buildInputs = [
              pkgs.mold
              rustToolchain
              self.checks.${system}.pre-commit-check.enabledPackages
              pkgs.openssl
              pkgs.clang
              pkgs.llvmPackages.bintools
              pkgs.gdb
            ];

            packages = with pkgs; [
              clang
              git-cliff
              nodejs
              pnpm
              upx
              coreutils
              rust-analyzer
              skopeo
              watchexec
              systemfd
              bacon
              openssl
              cargo-audit
              cargo-machete
              cargo-nextest
              cargo-insta
            ];
          }
        );
      }
    );
}
