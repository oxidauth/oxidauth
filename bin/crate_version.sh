#!/bin/bash
set -euo pipefail

# Print one workspace crate's version. The image build scripts derive each
# image tag from their crate's Cargo.toml (`src/oxidauth/oxidauth-api/build/
# build.sh` from plan 12), so a container's image tag and the crate's
# Cargo.toml are the same number read from the same place — Cargo.toml,
# through cargo's own metadata.
#
#   ./bin/crate_version.sh oxidauth-api      # 0.8.0
#   ./bin/crate_version.sh oxidauth-kernel   # 0.4.0

crate="${1:?usage: $(basename "$0") <crate>}"

cd "$(dirname "$0")/.."

# `|| true`: with pipefail, a missing crate makes the grep exit 1 and `set -e`
# would kill the script before the friendly error below could print.
version="$(cargo metadata --format-version 1 --no-deps |
    grep -o "\"name\":\"$crate\",\"version\":\"[^\"]*\"" |
    head -1 |
    sed -e 's/.*version":"//' -e 's/"$//')" || true

if [ -z "$version" ]; then
    echo "crate_version.sh: '$crate' is not a crate in this workspace" >&2
    echo "  members: $(cargo metadata --format-version 1 --no-deps |
        tr '{' '\n' |
        grep -o '"name":"[a-z0-9_-]*","version' |
        sed -e 's/.*"name":"//' -e 's/","version//' |
        tr '\n' ' ')" >&2
    exit 1
fi

echo "$version"
