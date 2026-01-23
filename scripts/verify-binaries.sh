#!/bin/bash
# Binary verification script
# Usage: ./scripts/verify-binaries.sh [release-tag]
# Example: ./scripts/verify-binaries.sh v0.1.0

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

TAG=${1:-latest}
REPO="jinali98/zk-profiling-solana"

echo -e "${GREEN}🔍 Verifying binaries for ${TAG}${NC}"

# Download checksums
echo -e "${YELLOW}Downloading checksums...${NC}"
if [ "$TAG" == "latest" ]; then
    CHECKSUMS_URL="https://github.com/${REPO}/releases/latest/download/checksums.txt"
else
    CHECKSUMS_URL="https://github.com/${REPO}/releases/download/${TAG}/checksums.txt"
fi

curl -L -o checksums.txt "${CHECKSUMS_URL}" || {
    echo -e "${RED}Error: Failed to download checksums${NC}"
    exit 1
}

# Verify each file
echo -e "${YELLOW}Verifying files...${NC}"
while IFS= read -r line; do
    if [ -z "$line" ]; then
        continue
    fi
    
    # Extract hash and filename
    HASH=$(echo "$line" | awk '{print $1}')
    FILENAME=$(echo "$line" | awk '{print $2}')
    
    if [ ! -f "$FILENAME" ]; then
        echo -e "${RED}✗ ${FILENAME} - File not found${NC}"
        continue
    fi
    
    # Calculate hash
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        CALCULATED_HASH=$(shasum -a 256 "$FILENAME" | awk '{print $1}')
    else
        # Linux
        CALCULATED_HASH=$(sha256sum "$FILENAME" | awk '{print $1}')
    fi
    
    # Compare
    if [ "$HASH" == "$CALCULATED_HASH" ]; then
        echo -e "${GREEN}✓ ${FILENAME} - OK${NC}"
    else
        echo -e "${RED}✗ ${FILENAME} - FAILED${NC}"
        echo -e "${RED}  Expected: ${HASH}${NC}"
        echo -e "${RED}  Got:      ${CALCULATED_HASH}${NC}"
        exit 1
    fi
done < checksums.txt

echo -e "${GREEN}✅ All binaries verified successfully!${NC}"
rm checksums.txt
