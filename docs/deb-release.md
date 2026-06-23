# Debian Package Release Guide

Use this checklist when building and publishing a Glypho `.deb` release.

The Debian package is defined in `flake.nix` as the `build_deb` package. It builds the Nix `glypho` package, copies the binary into `package/usr/bin`, writes a Debian `control` file, and runs `dpkg-deb`.

The package name, version, and description are read from `Cargo.toml`, so the `.deb` version follows the normal Cargo release version.

## Prerequisites

- You have Nix installed with flakes enabled
- You have a clean working tree: `git status --short`
- You have already updated `Cargo.toml`, `Cargo.lock`, and `CHANGELOG.md` for the release
- The Cargo release checks pass

## 1. Check Debian Metadata

Before building, check the `.deb` metadata in `flake.nix`.

Find the `deb-package` section and check:

- `Maintainer`
- `Section`
- `Priority`
- `Architecture`

The package `Version` and output filename are generated from `Cargo.toml`.

The current Debian package uses `Architecture: amd64`, so build this release from an x86_64 Linux environment.

## 2. Build the Package

From the repository root, run:

```sh
nix build .#build_deb
```

Nix writes the build output to `result`. The `.deb` file should be inside that output directory:

```sh
ls -lah result/
```

For version `0.2.1`, expect:

```text
result/glypho_0.2.1_amd64.deb
```

## 3. Inspect the Package

The `dpkg-deb` command is provided by Debian's `dpkg` package. If it is not installed on your system, enter the Nix dev shell first:

```sh
nix develop
```

Or run the inspection commands through a temporary Nix shell:

```sh
nix shell nixpkgs#dpkg
```

Check the package metadata:

```sh
dpkg-deb --info result/glypho_0.2.1_amd64.deb
```

Check the files included in the package:

```sh
dpkg-deb --contents result/glypho_0.2.1_amd64.deb
```

Confirm that the package contains:

```text
./usr/bin/glypho
```

## 4. Install and Test Locally

Install the package on a Debian or Ubuntu system:

```sh
sudo apt install ./result/glypho_0.2.1_amd64.deb
```

If you install a downloaded `.deb` from a private home directory and see a warning like `Download is performed unsandboxed as root`, the package can still install successfully. It means the `_apt` sandbox user cannot read the file path. To avoid the warning, copy the package to a world-readable location first:

```sh
cp ~/Downloads/glypho_0.2.1_amd64.deb /tmp/
sudo apt install /tmp/glypho_0.2.1_amd64.deb
```

You can also install directly with `dpkg`, but `apt install ./file.deb` is preferred when the package has dependencies because `apt` can resolve them.

Then verify the binary works:

```sh
glypho --version
glypho --help
```

If you need to remove the package:

```sh
sudo apt remove glypho
```

## 5. Commit the Release Files

If you changed the Debian metadata in `flake.nix`, commit it with the release changes:

```sh
git status --short
git add flake.nix Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore: release v0.2.1"
```

Add any other release-related files if they changed.

## 6. Tag and Push

Create and push the release tag:

```sh
git tag -a v0.2.1 -m "Release v0.2.1"
git push origin main
git push origin v0.2.1
```

If you release from a branch other than `main`, push that branch instead.

## 7. Publish the `.deb`

The `gh` command is GitHub's CLI. If it is not installed on your system, enter the Nix dev shell first:

```sh
nix develop
```

Or run the release command through a temporary Nix shell:

```sh
nix shell nixpkgs#gh
```

Make sure GitHub CLI is authenticated:

```sh
gh auth login
```

Attach the `.deb` file to the GitHub release for the tag:

```sh
gh release create v0.2.1 \
  result/glypho_0.2.1_amd64.deb \
  --title "Glypho v0.2.1" \
  --notes-file CHANGELOG.md
```

If the GitHub release already exists, upload the package to it:

```sh
gh release upload v0.2.1 result/glypho_0.2.1_amd64.deb
```

## 8. Final Checks

- Download the `.deb` from the GitHub release page
- Install the downloaded file on a clean Debian or Ubuntu system
- Confirm `glypho --version` reports the released version
- Confirm the release notes mention the `.deb` asset

## Troubleshooting

If the package version is wrong, update the version in `Cargo.toml`, then rebuild:

```sh
nix build .#build_deb
```

If `nix build .#build_deb` fails because frontend assets are stale, build the web assets first:

```sh
cd glypho-web
npm install
npm run build
cd ..
nix build .#build_deb
```
