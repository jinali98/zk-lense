# Deployment Guide

Simple commands to deploy a new version to GitHub Releases, crates.io, and Homebrew.

## Prerequisites

- GitHub repository access
- crates.io account with API token (run `cargo login YOUR_TOKEN` once)
- Homebrew tap repository cloned: `git clone https://github.com/gihanrcg/homebrew-zklense.git`

---

## Step 1: Update Version

Edit `cli/Cargo.toml`:
```toml
version = "0.1.2"  # Update to new version
```

---

## Step 2: Deploy to GitHub Releases

```bash
cd /Users/gihan/personal/hackspace/zkprof

# Commit version change
git add cli/Cargo.toml
git commit -m "Release v0.1.2"
git push origin master

# Create and push tag (triggers GitHub Actions)
git tag -a v0.1.2 -m "Release v0.1.2"
git push origin v0.1.2
```

**Wait for GitHub Actions to build and create release** (~10-20 minutes)

---

## Step 3: Deploy to crates.io

```bash
cd /Users/gihan/personal/hackspace/zkprof/cli

# Test first
cargo publish --dry-run

# Publish
cargo publish
```

---

## Step 4: Deploy to Homebrew

```bash
cd ~/homebrew-zklense

# Get SHA256 checksums from GitHub release page
# Or download and calculate:
curl -LO https://github.com/jinali98/zk-profiling-solana/releases/download/v0.1.2/zklense-x86_64-apple-darwin.tar.gz
curl -LO https://github.com/jinali98/zk-profiling-solana/releases/download/v0.1.2/zklense-aarch64-apple-darwin.tar.gz
shasum -a 256 zklense-x86_64-apple-darwin.tar.gz
shasum -a 256 zklense-aarch64-apple-darwin.tar.gz

# Edit Formula/zklense.rb:
# - Update version to "0.1.2"
# - Update URLs to v0.1.2
# - Update SHA256 values (remove "sha256:" prefix)

# Commit and push
git add Formula/zklense.rb
git commit -m "Update zklense to v0.1.2"
git push origin main
```

---

## Quick Reference

**GitHub:** `git tag -a v0.1.X -m "Release v0.1.X" && git push origin v0.1.X`

**crates.io:** `cd cli && cargo publish`

**Homebrew:** Update `Formula/zklense.rb` → commit → push

---

## Verify Installations

```bash
# GitHub Release
curl -LO https://github.com/jinali98/zk-profiling-solana/releases/download/v0.1.2/zklense-aarch64-apple-darwin.tar.gz
tar -xzf zklense-aarch64-apple-darwin.tar.gz
./zklense --version

# crates.io
cargo install zklense
zklense --version

# Homebrew
brew tap gihanrcg/zklense
brew install zklense
zklense --version
```



git clone https://github.com/reilabs/sunspot.git ~/sunspot

cd ~/sunspot/go && go build -o sunspot .

export PATH="$HOME/sunspot/go:$PATH"

export GNARK_VERIFIER_BIN="$HOME/sunspot/gnark-solana/crates/verifier-bin"

source ~/.zshrc