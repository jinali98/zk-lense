#!/bin/bash
# Release automation script for zklense
# Usage: ./scripts/release.sh [version]
# Example: ./scripts/release.sh 0.1.0

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get version from argument or prompt
if [ -z "$1" ]; then
    echo -e "${YELLOW}Enter version number (e.g., 0.1.0):${NC}"
    read -r VERSION
else
    VERSION=$1
fi

# Validate version format
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo -e "${RED}Error: Invalid version format. Use semantic versioning (e.g., 0.1.0)${NC}"
    exit 1
fi

TAG="v${VERSION}"
CLI_DIR="cli"

echo -e "${GREEN}🚀 Starting release process for version ${VERSION}${NC}"

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${RED}Error: Not in a git repository${NC}"
    exit 1
fi

# Check if working directory is clean
if ! git diff-index --quiet HEAD --; then
    echo -e "${RED}Error: Working directory is not clean. Commit or stash changes first.${NC}"
    exit 1
fi

# Check if tag already exists
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo -e "${RED}Error: Tag ${TAG} already exists${NC}"
    exit 1
fi

# Update version in Cargo.toml
echo -e "${YELLOW}📝 Updating version in Cargo.toml...${NC}"
sed -i.bak "s/^version = \".*\"/version = \"${VERSION}\"/" "${CLI_DIR}/Cargo.toml"
rm "${CLI_DIR}/Cargo.toml.bak"

# Build locally to verify
echo -e "${YELLOW}🔨 Building release binary...${NC}"
cd "${CLI_DIR}"
cargo build --release
cd ..

# Test the binary
echo -e "${YELLOW}🧪 Testing binary...${NC}"
if [ -f "${CLI_DIR}/target/release/zklense" ]; then
    "${CLI_DIR}/target/release/zklense" --version
else
    echo -e "${RED}Error: Binary not found${NC}"
    exit 1
fi

# Commit version change
echo -e "${YELLOW}📦 Committing version change...${NC}"
git add "${CLI_DIR}/Cargo.toml"
git commit -m "Release ${TAG}"

# Create and push tag
echo -e "${YELLOW}🏷️  Creating tag ${TAG}...${NC}"
git tag -a "${TAG}" -m "Release ${TAG}"

# Ask for confirmation before pushing
echo -e "${YELLOW}Ready to push tag ${TAG} to remote? (y/n)${NC}"
read -r CONFIRM
if [ "$CONFIRM" != "y" ] && [ "$CONFIRM" != "Y" ]; then
    echo -e "${YELLOW}Release preparation complete. Tag created locally.${NC}"
    echo -e "${YELLOW}To push manually: git push origin ${TAG}${NC}"
    exit 0
fi

# Push tag (this will trigger GitHub Actions)
echo -e "${GREEN}📤 Pushing tag to remote...${NC}"
git push origin "${TAG}"
git push origin master

echo -e "${GREEN}✅ Release ${TAG} initiated!${NC}"
echo -e "${GREEN}GitHub Actions will automatically build and create the release.${NC}"
echo -e "${YELLOW}Monitor progress at: https://github.com/jinali98/zk-lense/actions${NC}"
