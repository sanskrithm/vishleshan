# GitHub Repository Setup Checklist

Complete this checklist to set up Panini-RS on GitHub and enable all CI/CD workflows.

---

## Pre-GitHub Setup (Local)

- [ ] Clone repository locally
  ```bash
  git clone https://github.com/YOUR_ORG/panini-rs.git
  cd panini-rs
  ```

- [ ] Verify all code compiles
  ```bash
  cargo build --release
  ```

- [ ] Verify all tests pass
  ```bash
  cargo test --release
  ```

- [ ] Check Cargo.lock is committed
  ```bash
  git status | grep Cargo.lock
  ```

- [ ] Verify .gitignore exists and excludes `/target/`

- [ ] Commit all files
  ```bash
  git add .
  git commit -m "Initial commit: Panini-RS v0.2"
  ```

---

## GitHub Repository Creation

- [ ] Create new repository on GitHub
  - Repository name: `panini-rs`
  - Description: "Morphology-driven semantic compiler for Sanskrit data analytics"
  - Visibility: **Public** (for workflows to work)
  - Initialize with README: **No** (we have our own)
  - .gitignore: **Rust** (optional, we have one)
  - License: **MIT** (recommended)

- [ ] Copy repository URL
  ```
  https://github.com/YOUR_ORG/panini-rs.git
  ```

---

## Push to GitHub

- [ ] Add remote
  ```bash
  git remote add origin https://github.com/YOUR_ORG/panini-rs.git
  ```

- [ ] Rename branch to main (if needed)
  ```bash
  git branch -M main
  ```

- [ ] Push to GitHub
  ```bash
  git push -u origin main
  ```

- [ ] Verify push succeeded
  - Check GitHub repository page
  - Verify files appear in main branch

---

## Enable GitHub Actions

- [ ] Go to repository Settings
  - Click "Actions" in sidebar
  - Verify "Allow all actions and reusable workflows" is selected

- [ ] Verify workflows are visible
  - Click "Actions" tab on main page
  - Should see 5 workflows listed:
    - build.yml
    - release.yml
    - coverage.yml
    - docs.yml
    - security.yml

- [ ] Trigger first build manually (optional)
  ```bash
  git commit --allow-empty -m "Trigger CI"
  git push
  ```
  - Check Actions tab for running workflows

---

## Branch Protection Rules (Recommended)

- [ ] Go to Settings → Branches

- [ ] Add rule for `main` branch
  - Branch name: `main`
  - ✅ Require status checks to pass before merging
  - Select checks:
    - ✅ `build / test`
    - ✅ `build / clippy`
    - ✅ `build / fmt`
  - ✅ Dismiss stale pull request approvals

- [ ] Save changes

---

## Configure Secrets (Optional but Recommended)

- [ ] Go to Settings → Secrets and variables → Actions

- [ ] Add Codecov token (if using codecov.io)
  - [ ] Sign up at https://codecov.io/
  - [ ] Add GitHub repository
  - [ ] Get token
  - [ ] Create secret:
    - Name: `CODECOV_TOKEN`
    - Value: (paste token from codecov.io)
  - [ ] Save

- [ ] (Optional) Add Slack webhook for notifications
  - [ ] Get webhook URL from Slack
  - [ ] Create secret:
    - Name: `SLACK_WEBHOOK`
    - Value: (paste webhook)

- [ ] (Optional) Add Discord webhook
  - [ ] Get webhook URL from Discord
  - [ ] Create secret:
    - Name: `DISCORD_WEBHOOK`
    - Value: (paste webhook)

---

## Create First Release

- [ ] Create version tag locally
  ```bash
  git tag v0.2.0
  ```

- [ ] Push tag to GitHub
  ```bash
  git push origin v0.2.0
  ```

- [ ] Monitor Actions
  - Check Actions tab
  - Watch `release.yml` workflow run
  - Should take ~5 minutes total

- [ ] Verify Release Created
  - Go to "Releases" page
  - Should see `v0.2.0` release
  - Should see binary artifacts:
    - `panini-linux-x86_64-v0.2.0`
    - `panini-windows-x86_64-v0.2.0.exe`
    - `panini-macos-x86_64-v0.2.0`
    - `SHA256SUMS`

- [ ] Download and verify binary
  ```bash
  wget https://github.com/YOUR_ORG/panini-rs/releases/download/v0.2.0/panini-linux-x86_64-v0.2.0
  chmod +x panini-linux-x86_64-v0.2.0
  ./panini-linux-x86_64-v0.2.0 --help
  ```

---

## Enable GitHub Pages (Optional)

- [ ] Go to Settings → Pages

- [ ] Configure source
  - Source: Deploy from a branch
  - Branch: `gh-pages`
  - Folder: `/ (root)`

- [ ] Workflows will auto-deploy docs
  - Docs appear at: `https://YOUR_ORG.github.io/panini-rs/`

---

## Configure Issue Templates (Optional)

- [ ] Create `.github/ISSUE_TEMPLATE/` directory

- [ ] Create bug_report.md
  ```markdown
  ---
  name: Bug Report
  about: Report a bug
  ---
  
  ## Description
  ...
  ```

- [ ] Create feature_request.md
  ```markdown
  ---
  name: Feature Request
  about: Suggest a feature
  ---
  
  ## Description
  ...
  ```

---

## Setup Pull Request Template (Optional)

- [ ] Create `.github/PULL_REQUEST_TEMPLATE.md`
  ```markdown
  ## Description
  Closes #(issue)
  
  ## Changes
  - [ ] ...
  
  ## Testing
  - [ ] Tests pass
  - [ ] Docs updated
  ```

---

## Configure Repository Settings

### General

- [ ] Verify repository visibility: **Public**

- [ ] Default branch: **main**

- [ ] Description updated:
  ```
  Morphology-driven semantic compiler for Sanskrit data analytics
  ```

- [ ] Topics added (optional):
  ```
  compiler, rust, data-processing, query-optimizer,
  semantic-analysis, llvm, arrow, polars
  ```

### Code and Automation

- [ ] ✅ Automatically delete head branches
  - Keeps repository clean after PR merge

- [ ] ✅ Allow auto-merge
  - Simplify merging for contributors

- [ ] ✅ Allow squash merging
  - Cleaner commit history

### Security

- [ ] (Optional) Enable GitHub Advanced Security
  - Secret scanning
  - Dependency scanning
  - Code scanning

---

## Configure GitHub Pages Documentation (Optional)

- [ ] Create `.nojekyll` file (for GitHub Pages)
  ```bash
  touch .nojekyll
  ```

- [ ] Create `docs/` directory
  ```bash
  mkdir -p docs
  ```

- [ ] Add GitHub Pages workflow (or use existing build.yml)

---

## Verify All Workflows Work

### Build Workflow

- [ ] Commit a change
  ```bash
  git commit --allow-empty -m "Trigger build"
  git push
  ```

- [ ] Check Actions tab
  - ✅ `build / test` passed
  - ✅ `build / fmt` passed
  - ✅ `build / clippy` passed
  - ✅ `build / build` passed (all platforms)

### Release Workflow

- [ ] Create tag
  ```bash
  git tag v0.2.1
  git push origin v0.2.1
  ```

- [ ] Check Actions tab
  - ✅ `release / create-release` passed
  - ✅ `release / build-release` passed
  - ✅ `release / create-checksums` passed

- [ ] Check Releases page
  - ✅ New release v0.2.1 exists
  - ✅ All binaries present
  - ✅ SHA256SUMS present

### Coverage Workflow

- [ ] Commit to main
  ```bash
  git commit --allow-empty -m "Trigger coverage"
  git push
  ```

- [ ] Check Actions tab
  - ✅ `coverage / coverage` passed

- [ ] Check Codecov (if configured)
  - ✅ Coverage report visible

### Documentation Workflow

- [ ] Update a documentation file
  ```bash
  echo "# Updated" >> START_HERE.md
  git add START_HERE.md
  git commit -m "Update docs"
  git push
  ```

- [ ] Check Actions tab
  - ✅ `docs / docs` passed
  - ✅ `docs / check-docs` passed

### Security Workflow

- [ ] Check Actions tab
  - ✅ `security / security-audit` passed
  - ✅ `security / dependencies` completed

---

## Add Status Badges to README

- [ ] Update `README.md` with badges
  ```markdown
  # Panini-RS
  
  [![Build](https://github.com/YOUR_ORG/panini-rs/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/YOUR_ORG/panini-rs/actions/workflows/build.yml)
  [![Release](https://github.com/YOUR_ORG/panini-rs/actions/workflows/release.yml/badge.svg)](https://github.com/YOUR_ORG/panini-rs/actions/workflows/release.yml)
  [![Docs](https://github.com/YOUR_ORG/panini-rs/actions/workflows/docs.yml/badge.svg?branch=main)](https://github.com/YOUR_ORG/panini-rs/actions/workflows/docs.yml)
  ```

- [ ] Commit and push
  ```bash
  git add README.md
  git commit -m "Add status badges"
  git push
  ```

---

## Troubleshooting

### Workflows Not Showing in Actions Tab

**Problem:** Actions tab is empty

**Solution:**
1. Check Settings → Actions
2. Verify "Allow all actions" is selected
3. Verify `.github/workflows/` directory exists
4. Wait 5-10 seconds and refresh
5. Check for any syntax errors in YAML files

### Build Fails on First Push

**Problem:** Tests fail in CI but pass locally

**Potential causes:**
1. Different Rust version (use rust-toolchain.toml)
2. File path issues (use std::path::Path)
3. Line ending issues (configure .gitattributes)
4. Timing issues (flaky tests)

**Solution:**
1. Check error in Actions logs
2. Fix locally and commit
3. Re-push to trigger workflow

### Release Artifacts Not Uploading

**Problem:** Tag pushed but binaries not in release

**Solution:**
1. Verify tag format: `v0.2.0` (must start with `v`)
2. Check release.yml status in Actions tab
3. Look for error messages in workflow logs
4. Verify GitHub token has repo write access
5. Manually create release if needed

### Codecov Integration Not Working

**Problem:** Coverage not appearing in PRs

**Solution:**
1. Sign up at codecov.io
2. Add GitHub repository to codecov.io
3. Create CODECOV_TOKEN secret
4. Wait for next PR (codecov takes time to show up)

---

## Common Next Steps

- [ ] Create main branch protection rules
- [ ] Invite collaborators
- [ ] Create GitHub Discussions for feedback
- [ ] Set up issue labels (bug, feature, documentation, etc.)
- [ ] Add contributing guidelines (CONTRIBUTING.md)
- [ ] Add code of conduct (CODE_OF_CONDUCT.md)
- [ ] Add security policy (SECURITY.md)

---

## Documentation References

- **GitHub Actions Setup Guide**: `GITHUB_ACTIONS_GUIDE.md`
- **Workflow Documentation**: `.github/workflows/README.md`
- **Workflow Index**: `GITHUB_ACTIONS_INDEX.md`
- **GitHub Docs**: https://docs.github.com/
- **GitHub Actions Docs**: https://docs.github.com/en/actions

---

## Verification Checklist Summary

**Before first push:**
- ✅ Code compiles locally
- ✅ All tests pass locally
- ✅ `.github/workflows/` directory exists with all 5 files
- ✅ Git repository initialized and committed

**After creating GitHub repository:**
- ✅ Repository created as public
- ✅ Remote added locally
- ✅ Code pushed to main branch

**After first push:**
- ✅ Build workflow completes successfully
- ✅ All tests pass in CI
- ✅ Lint checks pass

**After creating first release:**
- ✅ Tag created and pushed
- ✅ Release workflow completes
- ✅ Binaries appear in GitHub Releases
- ✅ SHA256SUMS file present
- ✅ Binaries are executable

---

**Status: Ready for GitHub Setup**

Complete this checklist and your Panini-RS repository will be fully configured with production-grade CI/CD.

Questions? See `GITHUB_ACTIONS_GUIDE.md` or `.github/workflows/README.md`
