#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

# Script to build all services in oxidauth

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "running all build files..."
echo ""

export REGISTRY="registry.vizerapp.cloud/oxidauth"
export RUST_BASE_IMAGE_VERSION="v1.89.0"
export DEBIAN_BASE_IMAGE_VERSION="12.12"

# Find all build.sh files and execute them
# (oxidauth-api/build/build.sh ships with plan 12)
find ./src -type f -name "build.sh" -print0 | while IFS= read -r -d '' file; do
    echo -e "${YELLOW}================================================================================${NC}"
    echo -e "${GREEN}Executing: $file${NC}"

    # Get the directory containing the file
    file_dir=$(dirname "$file")

    # Make the file executable
    chmod +x "$file"

    # Execute the file from its directory
    (cd "$file_dir" && bash "./$(basename "$file")")

    exit_code=$?

    if [ $exit_code -eq 0 ]; then
        echo -e "${GREEN}✓ Successfully executed: $file${NC}"
    else
        echo -e "${RED}✗ Failed to execute: $file (exit code: $exit_code)${NC}"
    fi
    echo ""
    echo -e "${YELLOW}================================================================================${NC}"
done

echo ""
echo -e "${GREEN}All build.sh files have been processed.${NC}"
