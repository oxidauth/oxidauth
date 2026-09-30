#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

# Script to find and execute all hurl.sh files in nested directories

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "running all hurl.sh files..."
echo ""

# Find all hurl.sh files and execute them
find ./src -type f -name "hurl.sh" -print0 | while IFS= read -r -d '' file; do
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
echo -e "${GREEN}All hurl.sh files have been processed.${NC}"
