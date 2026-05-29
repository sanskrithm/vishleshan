# GitHub Actions CI/CD Setup for Panini-RS v0.2

This directory contains GitHub Actions workflows for automated building, testing, and releasing Panini-RS.

## Workflows

### 1. `build.yml` — Build and Test (Pull Requests & Commits)

**Triggers:**
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop` branches

**Jobs:**
- **Test Suite** (Ubuntu) — Runs all tests and doc tests
- **Rustfmt** (Ubuntu) — Checks code formatting
- **Clippy** (Ubuntu) — Runs linter checks
- **Build** (Multi-platform) — Builds for Linux, Windows, macOS

**Outputs:**
- Test results in GitHub Actions logs
- Build artifacts available for 5 days
- Linting feedback on pull requests

**Running Locally:**
```bash
cargo test --release --verbose
cargo fmt -- --check
cargo clippy --release -- -D warnings
cargo build --release
```

---

### 2. `release.yml` — Create Release Artifacts (Tags)

**Triggers:**
- Push of version tags (e.g., `v0.2.0`, `v0.2.1`)

**Jobs:**
- **Create Release** — Creates GitHub Release with changelog
- **Build Release** — Builds optimized binaries for all platforms
- **Create Checksums** — Generates SHA256 checksums for verification

**Outputs:**
- GitHub Release with binary artifacts
- Checksums for integrity verification
- Release notes

**Creating a Release:**
```bash
git tag v0.2.1
git push origin v0.2.1
```

---

### 3. `coverage.yml` — Code Coverage (Pull Requests & Commits)

**Triggers:**
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop` branches

**Jobs:**
- **Code Coverage** — Generates coverage report with tarpaulin

**Outputs:**
- Coverage metrics in Codecov
- Coverage trends visible on pull requests

**Running Locally:**
```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Xml --all-features --release
```

---

### 4. `docs.yml` — Documentation Build (Pull Requests & Main)

**Triggers:**
- Push to `main` branch
- Pull requests to `main` or `develop` branches

**Jobs:**
- **Build Documentation** — Generates Rust doc comments
- **Check Documentation** — Validates required docs exist

**Outputs:**
- Rust documentation buildable
- All required documentation files present

**Running Locally:**
```bash
cargo doc --no-deps --release
```

---

### 5. `security.yml` — Security Audit (Scheduled & On-Demand)

**Triggers:**
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop` branches
- Weekly scheduled (Sunday 00:00 UTC)

**Jobs:**
- **Security Audit** — Checks for known security vulnerabilities
- **Dependencies** — Lists outdated dependencies

**Outputs:**
- Security warnings in GitHub Actions logs
- Dependency update recommendations

**Running Locally:**
```bash
cargo install cargo-deny
cargo deny check advisories

cargo install cargo-outdated
cargo outdated
```

---

## Matrix Builds

### Supported Platforms

| OS | Target | Binary |
|-------|--------|--------|
| **Linux** | x86_64-unknown-linux-gnu | `panini-linux-x86_64` |
| **Windows** | x86_64-pc-windows-msvc | `panini-windows-x86_64.exe` |
| **macOS** | x86_64-apple-darwin | `panini-macos-x86_64` |

Each platform:
- ✅ Builds from source
- ✅ Runs all tests
- ✅ Creates optimized binary
- ✅ Strips debug symbols
- ✅ Uploads as artifact or release

---

## Environment Variables

All workflows use:

```yaml
CARGO_TERM_COLOR: always      # Colored cargo output
RUST_BACKTRACE: 1             # Full backtrace on panic
```

---

## Caching Strategy

Workflows use GitHub Actions caching for:

1. **Cargo Registry** (`~/.cargo/registry`) — Package metadata
2. **Cargo Index** (`~/.cargo/git`) — Git dependencies
3. **Build Cache** (`target/`) — Compiled artifacts

Cache keys:
```
${{ runner.os }}-cargo-{type}-${{ hashFiles('**/Cargo.lock') }}
```

This ensures:
- Fast subsequent builds (cache hit on unchanged dependencies)
- Automatic invalidation when `Cargo.lock` changes
- Per-OS caches (Linux, Windows, macOS separate)

---

## Release Process

### Step 1: Create Version Tag
```bash
git tag v0.2.1 -m "Release v0.2.1"
git push origin v0.2.1
```

### Step 2: GitHub Actions Triggers
- `release.yml` workflow automatically starts
- Creates GitHub Release
- Builds for all platforms
- Generates checksums
- Uploads artifacts

### Step 3: Verify Release
- Check GitHub Releases page
- Download binaries
- Verify checksums:
  ```bash
  sha256sum -c SHA256SUMS
  ```

---

## Artifact Management

### Build Artifacts (PR/Push)
- Stored for 5 days
- Deleted automatically after expiration
- Used for quick testing

### Release Artifacts (Tags)
- Stored indefinitely in GitHub Releases
- Versioned (e.g., `panini-linux-x86_64-v0.2.1`)
- Linked from releases page

---

## Failure Handling

### Test Failures
- Pull requests are blocked if tests fail
- Must fix before merge to `main`
- Logs show detailed error messages

### Build Failures
- Failures are reported in GitHub Actions
- Actions tab shows build logs
- Emails sent to commit author (if configured)

### Lint Failures
- Clippy warnings fail the build (`-D warnings`)
- All code must pass formatting checks
- Use `cargo fmt` to auto-fix formatting

---

## Local Development

To match CI behavior locally:

```bash
# Test
cargo test --release --verbose

# Format check
cargo fmt -- --check

# Lint
cargo clippy --release -- -D warnings

# Build
cargo build --release

# Full CI simulation
cargo test --release --verbose && \
cargo fmt -- --check && \
cargo clippy --release -- -D warnings && \
cargo build --release
```

---

## Secrets & Tokens

### GitHub Token
- Automatically provided by GitHub Actions
- Used for: creating releases, uploading artifacts
- No manual configuration needed

### Codecov Token (Optional)
- For codecov.io integration
- Can be added via Settings → Secrets

---

## Status Badges

Add to README.md:

```markdown
![Build Status](https://github.com/YOUR_ORG/panini-rs/actions/workflows/build.yml/badge.svg)
![Release Status](https://github.com/YOUR_ORG/panini-rs/actions/workflows/release.yml/badge.svg)
![Docs Status](https://github.com/YOUR_ORG/panini-rs/actions/workflows/docs.yml/badge.svg)
```

---

## Troubleshooting

### Cache not working?
- Check `Cargo.lock` hasn't changed unexpectedly
- Clear cache manually in GitHub Settings → Actions → Clear all caches

### Tests fail on Windows only?
- Check for path separators (`/` vs `\`)
- Use `std::path::Path` for cross-platform paths

### Build times too long?
- First build takes longer (cache miss)
- Subsequent builds use cache (much faster)
- Can parallelize independent workflows

### Release artifacts missing?
- Check `release.yml` status in Actions tab
- Verify tag is correctly formatted (`v*`)
- Check GitHub token has release permissions

---

## Performance

### Typical Times

| Job | Duration |
|-----|----------|
| Test Suite | ~60-90 sec |
| Clippy | ~40-60 sec |
| Linux Build | ~120-180 sec |
| Windows Build | ~150-210 sec |
| macOS Build | ~150-210 sec |

With caching:
- ~80% time savings on subsequent runs

---

## Next Steps

1. **Push to GitHub**
   ```bash
   git push origin main
   ```

2. **Check Actions Tab**
   - View workflow runs
   - Monitor build status

3. **Create Release**
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

4. **Download Artifacts**
   - Go to GitHub Releases
   - Download platform-specific binary

---

## Documentation

- **Workflows**: See YAML files in this directory
- **GitHub Actions**: https://docs.github.com/en/actions
- **Rust Actions**: https://github.com/dtolnay/rust-toolchain
- **Security**: https://github.com/EmbarkStudios/cargo-deny-action

---

**Panini-RS CI/CD Setup**  
Version: 0.2.0  
Date: May 29, 2026
