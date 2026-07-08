---
allowed-tools: Bash, Edit, Read, Glob
argument-hint: [version] (e.g., 0.1.0)
description: Automated release process - version bump, changelog, tag, publish
---

# Release Process for asciiquarium

## Pre-flight Checks

Current branch: !`git branch --show-current`

Last releases: !`git tag --sort=-version:refname | grep -v dev | head -5`

Current version: !`grep '^version' Cargo.toml | head -1`

## Steps

### 1. Version Bump

- Update `version` in `Cargo.toml` to $ARGUMENTS
- Run `cargo check` to update `Cargo.lock`

### 2. Generate Changelog

Get commits since last stable release:

```bash
last_tag=$(git tag --sort=-version:refname | grep -v dev | head -1)
git log --oneline --pretty=format:"* %s (%ad)" --date=short ${last_tag}..HEAD
```

Create `changes/v$ARGUMENTS.md` with:

- `# v$ARGUMENTS` header
- `## Highlights` section with notable user-facing changes
- `## Raw commits` section with the commit list
- **No soft line breaks** -- paragraphs should be single long lines. GitHub
  renders markdown with soft wraps, so hard breaks mid-paragraph show up as
  unwanted newlines in the release notes.

### 3. Review

**REVIEW REQUIRED**: Show the changelog for user approval before proceeding.

### 4. Verify

```bash
./scripts/check.sh
```

### 5. Commit and Tag

```bash
git add Cargo.toml Cargo.lock changes/v$ARGUMENTS.md
git commit -m "chore: release v$ARGUMENTS"
git tag v$ARGUMENTS
```

### 6. Push

```bash
git push && git push --tags
```

### 7. Cargo Publish

**Warning**: This step cannot be undone -- you cannot unpublish from crates.io.

```bash
cargo publish
```

### 8. Bump to Dev Version

Bump `Cargo.toml` to the next patch dev version (e.g. `0.1.0` -> `0.1.1-dev`),
run `cargo check`, and commit:

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: bump to v<next>-dev"
git push
```
