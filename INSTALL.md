# Installation & Deployment Guide

Complete guide for installing zklense CLI and deploying releases across platforms.

## Table of Contents

- [Installation Methods](#installation-methods)
  - [GitHub Releases](#github-releases-recommended)
  - [crates.io](#cratesio-rust-developers)
  - [Homebrew](#homebrew-macoslinux)
  - [Scoop](#scoop-windows)
  - [Build from Source](#build-from-source)
- [Platform-Specific Instructions](#platform-specific-instructions)
- [Verification](#verification)
- [Troubleshooting](#troubleshooting)
- [Deployment Guide](#deployment-guide)
  - [Creating GitHub Releases](#creating-github-releases)
  - [Publishing to crates.io](#publishing-to-cratesio)
  - [Updating Package Managers](#updating-package-managers)

---

## Installation Methods

### GitHub Releases (Recommended)

Pre-built binaries for all platforms are available on the [GitHub Releases](https://github.com/jinali98/zk-profiling-solana/releases) page.

#### Linux (x86_64)

```bash
# Download the latest release
curl -LO https://github.com/jinali98/zk-profiling-solana/releases/latest/download/zklense-x86_64-unknown-linux-gnu.tar.gz

# Extract
tar -xzf zklense-x86_64-unknown-linux-gnu.tar.gz

# Move to PATH
sudo mv zklense /usr/local/bin/

# Verify installation
zklense --version
```

#### macOS (Intel)

```bash
# Download the latest release
curl -LO https://github.com/jinali98/zk-profiling-solana/releases/latest/download/zklense-x86_64-apple-darwin.tar.gz

# Extract
tar -xzf zklense-x86_64-apple-darwin.tar.gz

# Move to PATH
sudo mv zklense /usr/local/bin/

# Verify installation
zklense --version
```

#### macOS (Apple Silicon)

```bash
# Download the latest release
curl -LO https://github.com/jinali98/zk-profiling-solana/releases/latest/download/zklense-aarch64-apple-darwin.tar.gz

# Extract
tar -xzf zklense-aarch64-apple-darwin.tar.gz

# Move to PATH
sudo mv zklense /usr/local/bin/

# Verify installation
zklense --version
```

#### Windows

```powershell
# Download the latest release
Invoke-WebRequest -Uri "https://github.com/jinali98/zk-profiling-solana/releases/latest/download/zklense-x86_64-pc-windows-msvc.zip" -OutFile "zklense.zip"

# Extract
Expand-Archive -Path zklense.zip -DestinationPath .

# Add to PATH (PowerShell)
$env:Path += ";$PWD"

# Or move to a permanent location
Move-Item zklense.exe C:\Program Files\zklense\
$env:Path += ";C:\Program Files\zklense\"

# Verify installation
zklense --version
```

---

### crates.io (Rust Developers)

If you have Rust installed, install directly from crates.io:

```bash
# Install from crates.io
cargo install zklense

# Verify installation
zklense --version
```

**Note:** This method compiles from source, so it may take a few minutes.

---

### Homebrew (macOS/Linux)

Install via Homebrew for easy updates:

```bash
# Add the tap
brew tap gihanrcg/zklense

# Install
brew install zklense

# Verify installation
zklense --version
```

**Updating:**

```bash
brew upgrade zklense
```

---

### Scoop (Windows)

Install via Scoop for Windows:

```powershell
# Add the bucket (if using a custom bucket)
scoop bucket add zklense https://github.com/YOUR_USERNAME/scoop-zklense.git

# Install
scoop install zklense

# Verify installation
zklense --version
```

**Updating:**

```powershell
scoop update zklense
```

---

### Build from Source

For development or custom builds:

#### Prerequisites

- **Rust** (1.70+): [Install Rust](https://rustup.rs/)
- **Git**: For cloning the repository

#### Build Steps

```bash
# Clone the repository
git clone https://github.com/jinali98/zk-profiling-solana.git
cd zkprof/cli

# Build release binary
cargo build --release

# The binary will be at target/release/zklense (or target/release/zklense.exe on Windows)
```

#### Add to PATH

**Linux/macOS:**

```bash
# Add to your shell profile (.bashrc, .zshrc, etc.)
export PATH="$PATH:/path/to/zkprof/cli/target/release"

# Or create a symlink
sudo ln -s /path/to/zkprof/cli/target/release/zklense /usr/local/bin/zklense
```

**Windows:**

```powershell
# Add to PATH permanently
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\path\to\zkprof\cli\target\release", [EnvironmentVariableTarget]::User)
```

---

## Platform-Specific Instructions

### Linux

**Dependencies:**

- `libssl-dev` (Ubuntu/Debian) or `openssl-devel` (Fedora/RHEL)
- `pkg-config`

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install libssl-dev pkg-config

# Fedora/RHEL
sudo dnf install openssl-devel pkg-config
```

### macOS

**Dependencies:**

- Xcode Command Line Tools

```bash
# Install Xcode Command Line Tools
xcode-select --install
```

### Windows

**Dependencies:**

- Visual C++ Build Tools (if building from source)
- Windows 10 or later

---

## Verification

### Verify Binary Integrity

All releases include SHA256 checksums. Verify your download:

**Linux/macOS:**

```bash
# Download checksums
curl -LO https://github.com/jinali98/zk-profiling-solana/releases/latest/download/checksums.txt

# Verify
sha256sum -c checksums.txt
```

**Windows (PowerShell):**

```powershell
# Download checksums
Invoke-WebRequest -Uri "https://github.com/jinali98/zk-profiling-solana/releases/latest/download/checksums.txt" -OutFile "checksums.txt"

# Verify
Get-FileHash zklense.exe -Algorithm SHA256 | Select-Object Hash
# Compare with checksums.txt
```

### Test Installation

```bash
# Check version
zklense --version

# Check help
zklense --help

# Initialize a test project
zklense init
```

---

## Troubleshooting

### Binary Not Found

**Problem:** `zklense: command not found`

**Solutions:**

1. **Check PATH:**
   ```bash
   # Linux/macOS
   echo $PATH
   
   # Windows
   echo %PATH%
   ```

2. **Verify binary location:**
   ```bash
   # Linux/macOS
   which zklense
   
   # Windows
   where zklense
   ```

3. **Re-add to PATH** (see installation instructions above)

### Permission Denied

**Problem:** `Permission denied` when running zklense

**Solutions:**

```bash
# Linux/macOS - Make executable
chmod +x zklense

# Or run with sudo (if needed)
sudo chmod +x /usr/local/bin/zklense
```

### Missing Dependencies

**Problem:** Runtime errors about missing libraries

**Solutions:**

- **Linux:** Install `libssl` and `libcrypto`
- **macOS:** Ensure Xcode Command Line Tools are installed
- **Windows:** Install Visual C++ Redistributable

### Build Errors (from source)

**Problem:** Compilation fails

**Solutions:**

1. **Update Rust:**
   ```bash
   rustup update
   ```

2. **Check Rust version:**
   ```bash
   rustc --version  # Should be 1.70+
   ```

3. **Clean and rebuild:**
   ```bash
   cargo clean
   cargo build --release
   ```

---

## Deployment Guide

This section is for maintainers who want to create releases and deploy zklense.

### Prerequisites for Deployment

- GitHub account with repository access
- Rust toolchain installed
- `cargo` and `cross` (for cross-compilation)
- GitHub CLI (`gh`) for releases (optional)

---

### Creating GitHub Releases

#### Automated Release (Recommended)

Releases are automatically created when you push a version tag:

```bash
# 1. Update version in cli/Cargo.toml
# version = "0.1.0"

# 2. Commit changes
git add cli/Cargo.toml
git commit -m "Release v0.1.0"

# 3. Create and push tag
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0

# GitHub Actions will automatically:
# - Build binaries for all platforms
# - Create a GitHub release
# - Upload binaries and checksums
```

#### Manual Release

If you need to create a release manually:

```bash
# 1. Build for all platforms (see scripts/release.sh)

# 2. Create release using GitHub CLI
gh release create v0.1.0 \
  --title "v0.1.0" \
  --notes "Release notes here" \
  zklense-*.tar.gz zklense-*.zip checksums.txt

# Or use GitHub web interface:
# 1. Go to Releases > New release
# 2. Select tag or create new one
# 3. Upload binaries and checksums
# 4. Publish release
```

---

### Publishing to crates.io

#### Prerequisites

1. **Create crates.io account:**
   - Visit [crates.io](https://crates.io)
   - Sign up with GitHub
   - Get your API token

2. **Login to cargo:**
   ```bash
   cargo login YOUR_API_TOKEN
   ```

#### Publishing Steps

```bash
# 1. Ensure Cargo.toml has all required metadata:
# - authors
# - description
# - license
# - repository
# - homepage

# 2. Dry run (check for issues)
cd cli
cargo publish --dry-run

# 3. Publish
cargo publish

# 4. Verify on crates.io
# Visit: https://crates.io/crates/zklense
```

**Important Notes:**

- Version must be incremented for each publish
- Published versions cannot be modified (only yanked)
- Ensure all dependencies are available on crates.io

---

### Updating Package Managers

#### Homebrew

**If using a custom tap (gihanrcg/homebrew-zklense):**

1. **Clone the tap repository:**
   ```bash
   git clone https://github.com/gihanrcg/homebrew-zklense.git
   cd homebrew-zklense
   ```

2. **Update formula** (`Formula/zklense.rb`):
   ```ruby
   version "0.1.0"
   url "https://github.com/jinali98/zk-profiling-solana/releases/download/v0.1.0/zklense-x86_64-apple-darwin.tar.gz"
   sha256 "CHECKSUM_HERE"
   ```

3. **Commit and push:**
   ```bash
   git add Formula/zklense.rb
   git commit -m "Update to v0.1.0"
   git push origin main
   ```

**If submitting to homebrew-core:**

- Follow [Homebrew submission guidelines](https://docs.brew.sh/Adding-Software-to-Homebrew)
- Create a pull request to [homebrew-core](https://github.com/Homebrew/homebrew-core)

#### Scoop

**If using a custom bucket:**

1. **Update manifest** (`.github/scoop-zklense.json`):
   ```json
   {
     "version": "0.1.0",
     "url": "https://github.com/jinali98/zk-profiling-solana/releases/download/v0.1.0/zklense-x86_64-pc-windows-msvc.zip",
     "hash": "CHECKSUM_HERE"
   }
   ```

2. **Commit and push:**
   ```bash
   git add scoop-zklense.json
   git commit -m "Update to v0.1.0"
   git push
   ```

**If submitting to main bucket:**

- Follow [Scoop submission guidelines](https://github.com/ScoopInstaller/Scoop/wiki/App-Manifests)
- Create a pull request to [ScoopInstaller/Main](https://github.com/ScoopInstaller/Main)

---

## Release Checklist

Before creating a release:

- [ ] Update version in `cli/Cargo.toml`
- [ ] Update CHANGELOG.md (if exists)
- [ ] Run tests: `cargo test`
- [ ] Build locally: `cargo build --release`
- [ ] Test binary: `./target/release/zklense --version`
- [ ] Create git tag: `git tag -a v0.1.0 -m "Release v0.1.0"`
- [ ] Push tag: `git push origin v0.1.0`
- [ ] Verify GitHub Actions build succeeds
- [ ] Verify release assets are uploaded
- [ ] Update package manager manifests (if applicable)
- [ ] Announce release (if applicable)

---

## Support

For issues or questions:

- **GitHub Issues:** [Create an issue](https://github.com/jinali98/zk-profiling-solana/issues)
- **Documentation:** See [README.md](README.md)
- **Discussions:** [GitHub Discussions](https://github.com/jinali98/zk-profiling-solana/discussions)

---

## License

See [LICENSE](LICENSE) file for details.
