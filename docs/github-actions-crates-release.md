# GitHub Actions crates.io Release Pipeline

This document explains how the `Publish crate to crates.io` GitHub Actions workflow publishes a new Glypho version.

The workflow lives at:

```text
.github/workflows/crates-release.yml
```

## Required Secret

The workflow publishes with a crates.io API token stored as a GitHub Actions secret:

```text
CRATES_IO_TOKEN
```

Create the token on crates.io with the smallest useful permission:

- Scope: `publish-update`
- Crate restriction: `glypho`

Then add it in GitHub under:

```text
Settings -> Secrets and variables -> Actions -> New repository secret
```

## Workflow Triggers

The workflow can start in three ways.

### 1. Version Change on `main`

When a commit is pushed to `main` and touches `Cargo.toml`, the workflow checks whether the package version changed.

Example version bump:

```toml
version = "0.2.1"
```

to:

```toml
version = "0.2.2"
```

If the version changed, the workflow runs release checks only. It does not publish to crates.io from a normal `main` push.

If `Cargo.toml` changed but the version did not, the workflow exits after printing a skip message.

### 2. Release Tag

When a tag like this is pushed:

```sh
git tag -a v0.2.2 -m "Release v0.2.2"
git push origin v0.2.2
```

the workflow treats it as the real crates.io release trigger.

The tag version must match `Cargo.toml`. For example, tag `v0.2.2` requires:

```toml
version = "0.2.2"
```

If the tag and `Cargo.toml` version do not match, the workflow fails before publishing.

### 3. Manual Run

The workflow can also be started from the GitHub Actions UI.

The manual input is:

```text
publish
```

Use `publish=false` to run release checks without publishing.

Use `publish=true` only when you intentionally want the workflow to run `cargo publish`.

## What the Workflow Checks

Before publishing, the workflow:

1. Checks out the repository.
2. Installs Rust with `rustfmt` and `clippy`.
3. Installs Node.js.
4. Runs `npm ci` in `glypho-web`.
5. Runs `npm run build`.
6. Copies `glypho-web/dist/index.html` to `src/template.html`.
7. Fails if `src/template.html` changed, because generated frontend output must be committed before release.
8. Runs:

   ```sh
   cargo fmt --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --locked
   cargo build --release --locked
   cargo publish --dry-run --locked
   ```

9. Publishes only if the run was triggered by a `v*` tag or manually with `publish=true`.

## Why the Frontend Build Runs in CI

Glypho embeds the frontend template into the Rust binary through `src/template.html`.

The workflow rebuilds the frontend to make sure `src/template.html` matches the current frontend source. If the rebuild changes the file, the release stops and asks you to commit the generated template first.

This keeps crates.io installs simple:

```sh
cargo install glypho
```

Users do not need Node.js or npm. The crate uses the committed `src/template.html`.

## Recommended Release Flow

Use this flow to avoid publishing from a commit that is not on `main`.

1. Update `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, and any generated files.
2. Commit the release:

   ```sh
   git add Cargo.toml Cargo.lock CHANGELOG.md src/template.html
   git commit -m "chore: release v0.2.2"
   ```

3. Push `main`:

   ```sh
   git push origin main
   ```

4. Wait for the version-change workflow run to pass.
5. Push the release tag:

   ```sh
   git tag -a v0.2.2 -m "Release v0.2.2"
   git push origin v0.2.2
   ```

6. Wait for the tag workflow run to publish the crate.
7. Confirm the new version appears on crates.io.

## If the Tag Workflow Fails

If the workflow fails before publishing, fix the issue and move the tag:

```sh
git tag -d v0.2.2
git push origin :refs/tags/v0.2.2
git tag -a v0.2.2 -m "Release v0.2.2"
git push origin v0.2.2
```

Only do this if crates.io did not accept the version.

Once a version is published to crates.io, it cannot be overwritten. Publish a new patch version instead.
