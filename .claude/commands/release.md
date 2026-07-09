---
allowed-tools: Bash, Edit, Read, Glob, WebFetch
argument-hint: [version] (e.g., 0.1.0)
description: Automated release process - version bump, changelog, verify, tag, publish
---

# Release Process for asciiquarium-rs

Execute the complete release workflow. Crate: `asciiquarium-rs`, command:
`asciiquarium-rs`, published to crates.io.

## Pre-flight Checks

Repository status: !`git status`

Current branch: !`git branch --show-current`

Last releases: !`git tag --sort=-version:refname | grep -v dev | head -5`

Current version: !`grep '^version' Cargo.toml | head -1`

## Steps

### 1. Confirm and Prerequisites

**Before starting, confirm with the user:**

- The version number: $ARGUMENTS
- That the crates.io token is set. If not, the user runs it themselves so the
  token never passes through the assistant: `! cargo login <token>`

### 2. Version Bump

- Update `version` in `Cargo.toml` to $ARGUMENTS (drop any `-dev` suffix)
- Run `cargo check` to update `Cargo.lock`

### 3. Verify

Everything must be green before tagging:

```bash
./scripts/check.sh   # cargo fmt --check, clippy -D warnings, tests
```

### 4. Generate Changelog

Get commits since the last stable release:

```bash
last_tag=$(git tag --sort=-version:refname | grep -v dev | head -1)
git log --oneline --pretty=format:"* %s (%ad)" --date=short ${last_tag}..HEAD
```

Create `changes/v$ARGUMENTS.md` with:

- `# v$ARGUMENTS` header
- `## Highlights` section focused on user-facing changes
- `## Raw commits` section with the commit list
- **No soft line breaks.** Paragraphs should be single long lines. GitHub
  renders markdown with soft wraps, so hard breaks mid-paragraph show up as
  unwanted newlines in the release notes.

### 5. Review Release Notes

**REVIEW REQUIRED**: Show `changes/v$ARGUMENTS.md` for approval. Do not proceed
until the user is satisfied that the highlights are accurate and user-facing.

### 6. Commit and Tag

```bash
git add Cargo.toml Cargo.lock changes/v$ARGUMENTS.md
git commit -m "chore: release v$ARGUMENTS"
git tag v$ARGUMENTS
git push && git push --tags
```

### 7. Distribution Prep (optional, only if the infra exists)

Skip anything the repo does not yet have.

- **CI binaries.** If a release workflow is present, watch it and confirm the
  GitHub release and its artifacts:
  ```bash
  gh run list --limit 1
  gh run watch <run-id> --exit-status
  gh release view v$ARGUMENTS --json body
  # if the body is just the commit message, set the real notes:
  gh release edit v$ARGUMENTS --notes-file changes/v$ARGUMENTS.md
  ```
- **Homebrew.** If a tap formula exists:
  - `cd ../homebrew-tap && git pull` first (never edit a stale checkout)
  - **Wait 10+ seconds** after the build for GitHub CDN propagation
  - Download the macOS tarball, verify it, and hash it:
    ```bash
    cd /tmp && rm -f asciiquarium-rs-v$ARGUMENTS-darwin-arm64.tar.gz
    curl -sL https://github.com/cablehead/asciiquarium-rs/releases/download/v$ARGUMENTS/asciiquarium-rs-v$ARGUMENTS-darwin-arm64.tar.gz -o asciiquarium-rs-v$ARGUMENTS-darwin-arm64.tar.gz
    tar -tzf asciiquarium-rs-v$ARGUMENTS-darwin-arm64.tar.gz   # verify contents
    sha256sum asciiquarium-rs-v$ARGUMENTS-darwin-arm64.tar.gz
    ```
  - Update the formula's version, URL, and SHA256; commit and push
  - **PAUSE** and have a macOS user verify before publishing:
    ```bash
    brew uninstall asciiquarium-rs 2>/dev/null || true
    brew install cablehead/tap/asciiquarium-rs
    asciiquarium-rs --version   # should show v$ARGUMENTS
    ```
    **STOP if verification fails.** crates.io is irreversible.

### 8. Cargo Publish

Package first, then publish:

```bash
cargo publish --dry-run
cargo publish
```

**Warning**: This cannot be undone. A version can be yanked but never
re-uploaded or deleted, so only publish once verification passes.

### 9. Bump to Dev Version

```bash
# Cargo.toml: 0.1.0 -> 0.1.1-dev
cargo check   # update Cargo.lock
git add Cargo.toml Cargo.lock
git commit -m "chore: bump to v<next>-dev"
git push
```

## Rollback Plan

If something fails **before `cargo publish`**:

1. Delete the tag: `git tag -d v$ARGUMENTS && git push --delete origin v$ARGUMENTS`
2. Delete the GitHub release (if created): `gh release delete v$ARGUMENTS`
3. Revert any homebrew formula changes
4. Fix the issue and retry

If `cargo publish` already ran, you cannot unpublish; yank it
(`cargo yank --version $ARGUMENTS`) and release a fixed patch version instead.

## Release Complete

- GitHub release: https://github.com/cablehead/asciiquarium-rs/releases/tag/v$ARGUMENTS
- Crates.io: `cargo install asciiquarium-rs`

---

**Ready to execute the release for v$ARGUMENTS?**
