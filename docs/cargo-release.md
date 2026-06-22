# Cargo Release Guide

Use this checklist when publishing a new Glypho release to crates.io.

## Prerequisites

- You have a clean working tree: `git status --short`
- You are logged in to crates.io: `cargo login`
- You have permission to publish the `glypho` crate
- You are releasing from the branch you want to tag, usually `main`

## 1. Choose the Version

Pick the next semantic version:

- Patch: bug fixes only, for example `0.2.1`
- Minor: new features, for example `0.3.0`
- Major: breaking changes, for example `1.0.0`

Update the version in `Cargo.toml`:

```toml
[package]
version = "0.3.0"
```

Then refresh the lockfile:

```sh
cargo check
```

## 2. Update Release Notes

Update `CHANGELOG.md` with the new version, release date, and notable changes.

This repository has `cliff.toml`, so you can generate changelog content with `git-cliff`.

If you use the Nix dev shell, `git-cliff` is already included:

```sh
nix develop
git-cliff --tag v0.3.0 --output CHANGELOG.md
```

Outside the Nix dev shell, run:

```sh
git-cliff --tag v0.3.0 --output CHANGELOG.md
```

If that prints `command not found: git-cliff`, install it with Cargo:

```sh
cargo install git-cliff
```

Then rerun the `git-cliff` command. Review the generated changelog before committing it.

## 3. Run Local Checks

Build the web assets first so the Rust release build can include the latest frontend output:

```sh
cd glypho-web
npm install
npm run build
cd ..
```

Then run the normal Rust checks from the repository root:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
```

## 4. Check Package Contents

Run a dry publish to confirm the crate packages correctly:

```sh
cargo publish --dry-run
```

Check the output for missing files, excluded files, or README/license issues.

If dependencies changed and you also ship the Flatpak manifest, refresh `cargo-sources.yaml` before publishing the Flatpak build.

## 5. Commit the Release

Commit the version, changelog, and lockfile changes:

```sh
git status --short
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore: release v0.3.0"
```

Add any other release-related files to the commit if they changed.

## 6. Publish to crates.io

Publish the crate:

```sh
cargo publish
```

Wait for crates.io to finish indexing, then verify the install works:

```sh
cargo install glypho --version 0.3.0
glypho --version
```

## 7. Tag and Push

Create and push the release tag:

```sh
git tag -a v0.3.0 -m "Release v0.3.0"
git push origin main
git push origin v0.3.0
```

If you release from a branch other than `main`, push that branch instead.

## 8. After Publishing

- Confirm the crate page shows the new version on crates.io
- Confirm the GitHub tag points to the release commit
- Create a GitHub release from the tag if you use GitHub releases
- Update any downstream packaging, including the Flatpak manifest, if needed

## Rollback Notes

Published crates cannot be deleted or overwritten. If a bad version is published:

1. Yank it so new builds avoid it:

   ```sh
   cargo yank --vers 0.3.0 glypho
   ```

2. Fix the issue.
3. Publish a new version, for example `0.3.1`.
