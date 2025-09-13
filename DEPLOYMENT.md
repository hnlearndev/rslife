# RSLife Deployment & Release Routine

## Deployment Process

### Package Publishing

- Published to [crates.io](https://crates.io/crates/rslife) (currently v0.2.9)
- Documentation auto-published to [docs.rs](https://docs.rs/rslife)
- Dual-licensed: Apache 2.0 & MIT

### CI/CD Pipeline (GitHub Actions)

| Workflow           | Purpose                                             |
| ------------------ | --------------------------------------------------- |
| `rust.yml`         | General CI (build + test across versions)           |
| `rust-stable.yml`  | Stable Rust: build, test, clippy lint, format check |
| `rust-1.89.0.yml`  | MSRV (Minimum Supported Rust Version) compatibility |
| `rust-nightly.yml` | Nightly Rust compatibility testing                  |
| `coverage.yml`     | Test coverage via grcov → Codecov                   |

### Quality Gates

- `cargo build --verbose`
- `cargo test --all --verbose`
- `cargo clippy --all -- -D warnings` (warnings as errors)
- `cargo fmt --all -- --check` (format enforcement)
- Test coverage tracking with Codecov

---

## Jujutsu (jj) Workflow

### Daily Development

```bash
# Create a new working copy / checkpoint
jj bookmark create my-feature      # jj bm create my-feature
jj bookmark set my-feature         # jj bm set my-feature

# Check status of changes
jj log -r '@'
jj status                          # jj st

# Amend last commit with new changes
jj amend                           # jj am

# Squash a series of commits into one
jj squash                          # jj sq
```

### Bookmark Management

```bash
# List all bookmarks
jj bookmark list                   # jj bm list

# Create a bookmark at current working copy
jj bookmark create feat-xyz        # jj bm create feat-xyz

# Move a bookmark to a specific revision
jj bookmark set feat-xyz -r <rev>  # jj bm set feat-xyz -r <rev>

# Delete a bookmark
jj bookmark delete feat-xyz        # jj bm delete feat-xyz
```

### Describe Commits

```bash
# Describe the current working copy commit
jj describe                        # jj desc / jj d

# Describe with a message inline
jj describe -m "feat: add table"   # jj d -m "feat: add table"

# Describe a specific revision
jj desc <rev> -m "fix: tuple order" # jj d <rev> -m "..."
```

### Push to GitHub

```bash
# Push the main branch
jj git push --branch main          # jj gpush -b main

# Push a specific bookmark (maps to a GitHub PR branch)
jj git push --bookmark feat-xyz    # jj gpush -B feat-xyz

# Push all bookmarks
jj git push --all                  # jj gpush -a

# Preview what will be pushed (dry run)
jj git push --dry-run              # jj gpush -n
```

### Sync from GitHub

```bash
# Fetch latest from remote
jj git fetch                       # jj gfetch

# Rebase your work on top of fetched main
jj rebase -r my-feature -d main    # jj rb -r my-feature -d main
```

---

## Pre-Publish Checklist

Before running `cargo publish`, execute this full routine:

### 1. Version Bump

```bash
# Update version in Cargo.toml
# Follow semver: MAJOR.MINOR.PATCH
```

### 2. Update Changelog / Release Notes

Ensure `README.md` reflects new features, breaking changes, or deprecations.

### 3. Full Test Suite

```bash
# Run all tests
 cargo test --all --verbose
```

### 4. Clippy Lint

```bash
# Treat warnings as errors
cargo clippy --all -- -D warnings
```

### 5. Format Check

```bash
# Check formatting
cargo fmt --all -- --check

# Auto-fix if needed
cargo fmt --all
```

### 6. Verify Documentation Builds

```bash
# Check that docs build without errors
cargo doc --no-deps --document-private-items
```

### 7. Run Examples

```bash
cargo run --example basic_usage
cargo run --example cm1_apr_2025
cargo run --example cm1_sep_2025
```

### 8. Commit & Describe in Jujutsu

```bash
jj describe -m "chore: bump version to 0.3.0"
jj git push --branch main
```

### 9. Dry-Run Publish

```bash
# Verify package contents before actual publish
cargo publish --dry-run
```

This checks:

- `Cargo.toml` is valid
- All files are included (use `cargo package --list` to inspect)
- No compilation or lint errors
- Package tarball is well-formed

### 10. Actual Publish

```bash
cargo publish
```

### 11. Verify on crates.io

Visit https://crates.io/crates/rslife to confirm:

- New version is listed
- README renders correctly
- Dependencies are resolved

### 12. Tag the Release (Optional)

```bash
jj git push --tag v0.3.0
```

Or via GitHub UI for release notes.
