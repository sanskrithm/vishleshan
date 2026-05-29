# GitHub Actions CI/CD Guide for Panini-RS v0.2

This guide explains the complete GitHub Actions setup for automated building, testing, and releasing Panini-RS.

## Quick Start

### For Repository Maintainers

1. **Push repository to GitHub**
   ```bash
   git remote add origin https://github.com/YOUR_ORG/panini-rs.git
   git push -u origin main
   ```

2. **Workflows automatically start**
   - Build workflow triggers on push to `main`
   - Tests run automatically
   - Artifacts appear in GitHub Actions tab

3. **Create a release**
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

4. **Download release artifacts**
   - Navigate to GitHub Releases page
   - Download platform-specific binary (Linux/Windows/macOS)

### For Contributors

- **Push commits** → automated tests run
- **Create PR** → tests must pass before merge
- **Merge to main** → builds are cached for speed

---

## Workflow Architecture

```
┌─────────────────────────────────────────────────────────┐
│        GitHub Actions CI/CD Pipeline                    │
└─────────────────────────────────────────────────────────┘

PUSH/PR EVENT
    ↓
    ├─→ build.yml (runs on every push/PR)
    │   ├─→ Test Suite (Ubuntu) ✓
    │   ├─→ Rustfmt (Ubuntu) ✓
    │   ├─→ Clippy (Ubuntu) ✓
    │   └─→ Build Matrix (Linux/Windows/macOS) ✓
    │
    ├─→ coverage.yml (code coverage)
    │   └─→ Tarpaulin Coverage → Codecov ✓
    │
    └─→ docs.yml (documentation check)
        ├─→ Build Docs ✓
        └─→ Check Files ✓

TAG PUSH EVENT (v0.2.0)
    ↓
    └─→ release.yml (only on tags)
        ├─→ Create Release ✓
        ├─→ Build Release Binaries ✓
        └─→ Generate Checksums ✓

SCHEDULED
    ↓
    └─→ security.yml (weekly + on push)
        ├─→ Security Audit ✓
        └─→ Dependency Check ✓
```

---

## Workflow Details

### 1. Build Workflow (`build.yml`)

**Triggers:**
- Every push to `main` or `develop`
- Every pull request to `main` or `develop`

**Jobs:**

#### Job 1: Test Suite
```yaml
runs-on: ubuntu-latest
steps:
  - Check out code
  - Install Rust
  - Run: cargo test --release --verbose
  - Run: cargo test --doc --release
```

**What it checks:**
- Unit tests pass
- Doc tests pass
- No panics

**Failure behavior:**
- PR is marked as failing
- Cannot merge to main
- Logs show which tests failed

---

#### Job 2: Rustfmt (Code Formatting)
```yaml
runs-on: ubuntu-latest
steps:
  - Check out code
  - Install Rust
  - Run: cargo fmt -- --check
```

**What it checks:**
- Code follows Rust formatting standards
- Consistent indentation
- Proper spacing

**Failure behavior:**
- Build fails
- Use `cargo fmt` locally to fix:
  ```bash
  cargo fmt
  git add .
  git commit -m "Format code"
  git push
  ```

---

#### Job 3: Clippy Linter
```yaml
runs-on: ubuntu-latest
steps:
  - Check out code
  - Install Rust
  - Run: cargo clippy --release -- -D warnings
```

**What it checks:**
- Common Rust mistakes
- Performance issues
- Code quality

**Failure behavior:**
- Build fails if any warnings exist
- Fix with:
  ```bash
  cargo clippy --fix --allow-dirty --allow-staged
  ```

---

#### Job 4: Multi-Platform Build
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, windows-latest, macos-latest]
steps:
  - Check out code
  - Install Rust
  - Build: cargo build --release
  - Strip binary (Unix only)
  - Upload artifact
```

**Builds for:**
1. **Linux** (Ubuntu latest)
   - Target: `x86_64-unknown-linux-gnu`
   - Binary: `target/release/panini`
   - Artifact: `panini-linux-x86_64`

2. **Windows** (Windows latest)
   - Target: `x86_64-pc-windows-msvc`
   - Binary: `target/release/panini.exe`
   - Artifact: `panini-windows-x86_64.exe`

3. **macOS** (macOS latest)
   - Target: `x86_64-apple-darwin`
   - Binary: `target/release/panini`
   - Artifact: `panini-macos-x86_64`

**Artifacts:**
- Stored for 5 days
- Downloadable from GitHub Actions → workflow run
- Used for quick testing

---

### 2. Release Workflow (`release.yml`)

**Triggers:**
- Only on git tags matching `v*` (e.g., `v0.2.0`, `v0.2.1`)

**How to trigger:**
```bash
git tag v0.2.0
git push origin v0.2.0
```

**Jobs:**

#### Job 1: Create Release
```yaml
Create GitHub Release entry with:
  - Release title: "Panini-RS v0.2.0"
  - Release body: (auto-populated)
  - Upload URL: saved for next job
```

#### Job 2: Build Release Binaries
```yaml
For each platform (Linux/Windows/macOS):
  - Checkout code
  - Install Rust
  - Build: cargo build --release
  - Strip debug symbols (Unix only)
  - Upload to release with naming:
    * panini-linux-x86_64-v0.2.0
    * panini-windows-x86_64-v0.2.0.exe
    * panini-macos-x86_64-v0.2.0
```

#### Job 3: Generate Checksums
```yaml
For each uploaded binary:
  - Generate SHA256 hash
  - Create: SHA256SUMS file
  - Upload to release
```

**Example Release Page:**
```
Release v0.2.0 · May 29, 2026
Assets:
  ✓ panini-linux-x86_64-v0.2.0 (5.2 MB)
  ✓ panini-windows-x86_64-v0.2.0.exe (5.5 MB)
  ✓ panini-macos-x86_64-v0.2.0 (5.1 MB)
  ✓ SHA256SUMS (156 bytes)
```

**Verify Integrity:**
```bash
# Download SHA256SUMS and binary, then:
sha256sum -c SHA256SUMS

# Output:
# panini-linux-x86_64-v0.2.0: OK
```

---

### 3. Code Coverage Workflow (`coverage.yml`)

**Triggers:**
- Every push to `main` or `develop`
- Every pull request to `main` or `develop`

**Job: Code Coverage**
```yaml
steps:
  - Install Rust
  - Install tarpaulin: cargo install cargo-tarpaulin
  - Generate coverage: cargo tarpaulin --out Xml --all-features
  - Upload to codecov.io
```

**What it measures:**
- Which code paths are tested
- Test coverage percentage
- Coverage trends over time

**Results:**
- Available on Codecov dashboard
- Coverage badge available for README

---

### 4. Documentation Workflow (`docs.yml`)

**Triggers:**
- Every push to `main` branch
- Every pull request to `main` or `develop`

**Jobs:**

#### Job 1: Build Documentation
```yaml
steps:
  - Generate: cargo doc --no-deps --release
  - Check: RUSTDOCFLAGS="-D warnings" (fail on warnings)
```

#### Job 2: Check Documentation Files
```yaml
steps:
  - Verify existence of:
    * START_HERE.md
    * QUICKREF_v0.2.md
    * README.md
    * SEMANTIC_GRAPH_ARCHITECTURE.md
    * v0.2_CONSOLIDATION_COMPLETE.md
```

---

### 5. Security Workflow (`security.yml`)

**Triggers:**
- Every push to `main` or `develop`
- Every pull request to `main` or `develop`
- Weekly scheduled (Sunday 00:00 UTC)

**Jobs:**

#### Job 1: Security Audit
```yaml
steps:
  - Install cargo-deny
  - Run: cargo deny check advisories
  - Checks against known CVE database
```

#### Job 2: Dependency Check
```yaml
steps:
  - Install cargo-outdated
  - List outdated dependencies
  - Suggests updates
```

---

## Caching Strategy

### How Caching Works

```
First Build:
  ├─→ Download dependencies (~30 sec)
  ├─→ Compile dependencies (~60 sec)
  ├─→ Compile project (~30 sec)
  └─→ Total: ~120 sec

Subsequent Builds (with cache):
  ├─→ Restore cache (~5 sec)
  ├─→ Compile project (~10 sec)
  └─→ Total: ~15 sec

Speed improvement: 8x faster!
```

### Cache Keys

All workflows use:
```yaml
key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
restore-keys: |
  ${{ runner.os }}-cargo-
```

**When cache invalidates:**
- `Cargo.lock` changes (dependencies updated)
- Different OS (Linux/Windows/macOS)
- 5 days of inactivity (GitHub limit)

### What Gets Cached

1. **Cargo Registry** (~10 MB)
   - Package metadata
   - Source code of dependencies

2. **Cargo Git** (~5 MB)
   - Git dependencies
   - Index metadata

3. **Build Cache** (~100-500 MB)
   - Compiled object files
   - Generated code

---

## Environment Variables

Defined in workflows:

```yaml
CARGO_TERM_COLOR: always  # Colored output
RUST_BACKTRACE: 1         # Full backtraces
```

**For sensitive data:** Use GitHub Secrets (Settings → Secrets)

---

## Release Versioning

### Current Version
```toml
# In Cargo.toml
[package]
name = "panini-rs"
version = "0.2.0"
```

### Versioning Strategy (Semantic Versioning)

```
v0.2.0
 ↓ ↓ ↓
 │ │ └─→ Patch (bug fixes)
 │ └───→ Minor (features, backward compatible)
 └─────→ Major (breaking changes)

Examples:
  v0.2.0 → v0.2.1 (patch)
  v0.2.1 → v0.3.0 (minor)
  v0.3.0 → v1.0.0 (major)
```

### Creating Releases

```bash
# For patch release (0.2.0 → 0.2.1)
git tag v0.2.1
git push origin v0.2.1

# For minor release (0.2.1 → 0.3.0)
git tag v0.3.0
git push origin v0.3.0
```

---

## Common Issues & Solutions

### Issue: Tests fail on Windows only

**Problem:**
- Path separators differ (`/` vs `\`)
- Line endings differ (`\n` vs `\r\n`)

**Solution:**
```rust
use std::path::Path;

// Instead of:
let path = "path/to/file";

// Use:
let path = Path::new("path/to/file");
```

---

### Issue: Build times are slow

**Problem:**
- First build has cache miss
- Incremental builds help

**Solution:**
- Wait for cache to build (normal)
- Subsequent commits use cache (faster)
- Run locally: `cargo build --release`

---

### Issue: Release artifacts not appearing

**Problem:**
- Tag format incorrect
- Workflow didn't trigger

**Solution:**
```bash
# Must start with 'v'
git tag v0.2.0    # ✓ Correct
git tag 0.2.0     # ✗ Incorrect (won't trigger)

# Verify tag exists
git tag -l

# Push tag to remote
git push origin v0.2.0
```

---

### Issue: Codecov integration not working

**Problem:**
- No codecov.io account
- Token not configured

**Solution:**
1. Sign up at codecov.io
2. Add to GitHub Secrets:
   - Settings → Secrets → New
   - Name: `CODECOV_TOKEN`
   - Value: (from codecov.io)
3. Update workflow to use token:
   ```yaml
   - uses: codecov/codecov-action@v3
     with:
       token: ${{ secrets.CODECOV_TOKEN }}
   ```

---

## Performance Tuning

### Parallelize Independent Jobs

All build workflow jobs run in **parallel**:

```
┌──────────────────────┐
│                      │
├─→ Tests    (~90 sec)
├─→ Rustfmt  (~50 sec)
├─→ Clippy   (~60 sec)
├─→ Linux    (~150 sec)
├─→ Windows  (~150 sec)
└─→ macOS    (~150 sec)

Total time: ~150 sec (longest job)
NOT: 90+50+60+150+150+150 = 650 sec
```

### Cache Warmth

```
1st push:   ~150 sec (no cache)
2nd push:   ~50 sec (cache hit)
3rd push:   ~50 sec (cache warm)
```

### Optimization Tips

1. **Keep dependencies minimal** → faster builds
2. **Use workspace** → better caching
3. **Incremental builds** → cache helps
4. **Parallel jobs** → already configured

---

## Monitoring & Debugging

### View Workflow Runs

1. Navigate to GitHub repository
2. Click "Actions" tab
3. Select workflow name
4. Click run to see details

### View Build Logs

1. Click failing job
2. Expand steps to see full output
3. Search for error messages

### Enable Debug Logging

```yaml
# In workflow file
env:
  RUST_LOG: debug
  RUST_BACKTRACE: full
```

---

## Security Best Practices

### 1. No Secrets in Code
```rust
// ✗ Wrong
const API_KEY: &str = "sk_live_xyz";

// ✓ Right
let api_key = std::env::var("API_KEY")?;
```

### 2. GitHub Secrets
- Settings → Secrets → Actions
- Only available during workflow execution
- Never logged or displayed

### 3. Dependabot
- GitHub automatically checks dependencies
- Security audit runs weekly
- Alerts on vulnerabilities

---

## Integration with Other Tools

### GitHub Pages (Documentation Hosting)

```yaml
# Optional: Publish docs to GitHub Pages
- name: Deploy docs
  uses: peaceiris/actions-gh-pages@v3
  with:
    github_token: ${{ secrets.GITHUB_TOKEN }}
    publish_dir: ./target/doc
```

### Discord/Slack Notifications

```yaml
# Optional: Notify on failures
- name: Notify Slack
  if: failure()
  uses: slackapi/slack-github-action@v1
  with:
    webhook-url: ${{ secrets.SLACK_WEBHOOK }}
    payload: |
      { "text": "Build failed!" }
```

### Status Badges

Add to README.md:

```markdown
# Panini-RS

![Build](https://github.com/YOUR_ORG/panini-rs/actions/workflows/build.yml/badge.svg?branch=main)
![Release](https://github.com/YOUR_ORG/panini-rs/actions/workflows/release.yml/badge.svg)
![Docs](https://github.com/YOUR_ORG/panini-rs/actions/workflows/docs.yml/badge.svg)
```

---

## Advanced Topics

### Cross-Compilation

Currently supports:
- Linux x86_64
- Windows x86_64
- macOS x86_64

To add ARM64:

```yaml
strategy:
  matrix:
    include:
      - os: ubuntu-latest
        target: aarch64-unknown-linux-gnu
      - os: macos-latest
        target: aarch64-apple-darwin
```

### Self-Hosted Runners

For private builds or specific hardware:

```yaml
runs-on: self-hosted
# Requires GitHub Actions Runner installed
```

### Scheduled Jobs

```yaml
on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM UTC
```

---

## Reference

- **GitHub Actions Docs**: https://docs.github.com/en/actions
- **Rust Actions**: https://github.com/dtolnay/rust-toolchain
- **Cargo Deny**: https://embarkstudios.github.io/cargo-deny/
- **Codecov**: https://codecov.io/
- **Semantic Versioning**: https://semver.org/

---

## Summary

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `build.yml` | Push/PR | Test, lint, build |
| `release.yml` | Tags (v*) | Create releases, binaries |
| `coverage.yml` | Push/PR | Code coverage tracking |
| `docs.yml` | Push/PR | Documentation checks |
| `security.yml` | Weekly/Push/PR | Security audits |

**Status:** ✅ Complete and ready for production use

**Next Step:** Push repository to GitHub and workflows will start automatically.

---

**Panini-RS GitHub Actions Setup**  
Version: 0.2.0  
Date: May 29, 2026
