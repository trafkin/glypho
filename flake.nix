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

        pkgs = import nixpkgs {inherit system overlays;};

        isLinux = pkgs.stdenv.isLinux;

        # mold is an ELF linker: Linux only. On Darwin the default ld64/lld
        # toolchain is used instead.
        stdenv =
          if isLinux
          then pkgs.stdenvAdapters.useMoldLinker pkgs.stdenv
          else pkgs.stdenv;

        rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        craneLib = (crane.mkLib pkgs).overrideScope (final: prev: {
          stdenvSelector = p: p.clangStdenv;
        });

        cargoPackage = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package;

        # Target triple per host system. Static musl is the Linux default;
        # Darwin builds use the native apple-darwin target (musl does not
        # exist for macOS).
        arch =
          if pkgs.stdenv.hostPlatform.isAarch64
          then "aarch64"
          else "x86_64";
        cargoTarget =
          if isLinux
          then "${arch}-unknown-linux-musl"
          else "${arch}-apple-darwin";

        # crt-static only makes sense for musl; mold only links ELF.
        cargoRustFlags = pkgs.lib.optionals isLinux [
          "-C target-feature=+crt-static"
          "-C linker=clang"
          "-C link-arg=-fuse-ld=${pkgs.mold}/bin/mold"
        ];

        src = let
          unfilteredRoot = ./.; # The original, unfiltered source
          files = pkgs.lib.fileset.unions [
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            (pkgs.lib.fileset.fileFilter (file: file.hasExt "html") unfilteredRoot)
            (pkgs.lib.fileset.fileFilter (file: file.hasExt "snap") unfilteredRoot)
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

        # Frontend dependencies, prefetched as a fixed-output derivation so
        # the sandboxed build can populate node_modules offline. When
        # glypho-web/package-lock.json changes, update this hash (set it to
        # pkgs.lib.fakeHash and rebuild to get the correct value).
        npmDeps = pkgs.fetchNpmDeps {
          src = ./glypho-web;
          hash = "sha256-6jdKqsWod7igG22sjAOjkNBCnhuwDnQqWrd+2DwAEow=";
        };

        commonArgs =
          {
            inherit src;
            CARGO_BUILD_TARGET = cargoTarget;
            buildInputs = with pkgs;
              [
                rustToolchain
                nodejs
              ]
              ++ pkgs.lib.optionals isLinux [
                mold
                openssl
              ];
            nativeBuildInputs = with pkgs;
              [
                rustToolchain
                clang
                pkg-config
              ]
              ++ pkgs.lib.optionals isLinux [
                mold
                upx
                llvmPackages.bintools
                gdb
              ];
          }
          // pkgs.lib.optionalAttrs isLinux {
            CARGO_BUILD_RUSTFLAGS = pkgs.lib.concatStringsSep " " cargoRustFlags;
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
            OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
            OPENSSL_LIB_DIR = "${pkgs.lib.getLib pkgs.openssl}/lib";
          };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        glypho = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts npmDeps;
            doCheck = false;
            pname = "glypho";
            # The frontend lives in a subdirectory; the npmConfigHook looks
            # there for package-lock.json and installs node_modules offline
            # from the prefetched npmDeps cache.
            npmRoot = "glypho-web";
            nativeBuildInputs =
              commonArgs.nativeBuildInputs
              ++ (with pkgs; [
                nodejs
                npmHooks.npmConfigHook
              ]);
            # The sandbox has no /usr/bin/env; rewrite the shebangs of the
            # npm-cli shims (installed by npmConfigHook) so build.rs can run
            # `npm run build`.
            preBuild = ''
              patchShebangs glypho-web/node_modules
            '';
            # UPX handles ELF reliably; Mach-O compression is unsupported here.
            postInstall = pkgs.lib.optionalString isLinux ''
              ${pkgs.upx}/bin/upx $out/bin/${glypho.pname}
            '';
          }
        );

        deb-package = pkgs.stdenv.mkDerivation {
          name = "${cargoPackage.name}-${cargoPackage.version}-deb";
          src = ./.;

          nativeBuildInputs = [pkgs.dpkg];
          buildInputs = [glypho];

          buildPhase = ''
            mkdir -p package/usr/bin
            mkdir -p package/DEBIAN

            cp ${glypho}/bin/glypho package/usr/bin/

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

        runCargoTests = craneLib.cargoTest (commonArgs
          // {
            inherit src cargoArtifacts npmDeps;
            npmRoot = "glypho-web";
            nativeBuildInputs =
              commonArgs.nativeBuildInputs
              ++ (with pkgs; [
                nodejs
                npmHooks.npmConfigHook
              ]);
            preBuild = ''
              patchShebangs glypho-web/node_modules
            '';
          });
      in {
        packages =
          {
            inherit glypho runCargoTests;
            default = glypho;
          }
          // pkgs.lib.optionalAttrs isLinux {
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
          craneLib.devShell
          {
            inherit (self.checks.${system}.pre-commit-check) shellHook;
            CARGO_BUILD_TARGET = cargoTarget;

            nativeBuildInputs = with pkgs;
              [
                rustToolchain
                sccache
                pkg-config
                clang
              ]
              ++ pkgs.lib.optionals isLinux [
                mold
                llvmPackages.bintools
              ];

            buildInputs = with pkgs;
              [
                rustToolchain
                clang
              ]
              ++ self.checks.${system}.pre-commit-check.enabledPackages
              ++ pkgs.lib.optionals isLinux [
                mold
                openssl
                llvmPackages.bintools
                gdb
              ];

            packages = with pkgs;
              [
                clang
                git-cliff
                gh
                nodejs
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
              ++ pkgs.lib.optionals isLinux [
                dpkg
                upx
                openssl
              ];
          }
          // pkgs.lib.optionalAttrs isLinux {
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
            OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
            OPENSSL_LIB_DIR = "${pkgs.lib.getLib pkgs.openssl}/lib";
            CARGO_BUILD_RUSTFLAGS = pkgs.lib.concatStringsSep " " cargoRustFlags;
          }
        );
      }
    );
}
