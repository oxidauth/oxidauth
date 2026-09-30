#!/bin/bash
set -euo pipefail

# Build (and optionally push) the multi-arch production image for
# oxidauth-api. Run directly or via the repo-root bin/build.sh, which
# exports REGISTRY / RUST_BASE_IMAGE_VERSION / DEBIAN_BASE_IMAGE_VERSION.
#
#   PUSH=1 ./build.sh     # build linux/amd64,linux/arm64 and push the
#                         # manifest list (semver + :latest + git-sha tags)
#   ./build.sh            # DEFAULT: the same multi-arch build, NO push —
#                         # exported as an OCI tarball under /tmp. The push
#                         # to registry.vizerapp.cloud is an explicit human
#                         # gate (plan 12).
#
# The build context is the repo root (the whole cargo workspace), so this
# script works from anywhere: the root is derived from the script's own
# location, never from $PWD.

NAME=oxidauth-api

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"

# image tag == crate version (bin/crate_version.sh reads it from Cargo.toml
# through cargo metadata); the git short SHA is the deployment-identity tag
# (semver+git pair preserved from the old bin/build-server.sh).
VERSION="$(bash "$REPO_ROOT/bin/crate_version.sh" "$NAME")"
GIT_VERSION="$(git -C "$REPO_ROOT" rev-parse --short HEAD)"
REGISTRY="${REGISTRY:-registry.vizerapp.cloud/oxidauth}"
IMAGE="$REGISTRY/$NAME"
PLATFORMS="${DOCKER_PLATFORMS:-linux/amd64,linux/arm64}"
BUILDER=oxidauth-builder
PUSH="${PUSH:-0}"
RUST_BASE_IMAGE_VERSION="${RUST_BASE_IMAGE_VERSION:-v1.89.0}"
DEBIAN_BASE_IMAGE_VERSION="${DEBIAN_BASE_IMAGE_VERSION:-12.12}"

export DOCKER_BUILDKIT=1

# the default docker driver can't push (or export) a multi-platform manifest,
# so builds run on a dedicated docker-container builder; binfmt registers QEMU
# so the host can execute the foreign-arch stages (Docker Desktop already
# ships it). Reuse the builder when it already exists (parkinglot pattern).
if ! docker buildx inspect "$BUILDER" > /dev/null 2>&1; then
  echo "creating multi-platform builder $BUILDER"
  docker buildx create --name "$BUILDER" --driver docker-container
  docker run --privileged --rm tonistiigi/binfmt --install all
fi

echo "building $NAME $VERSION ($GIT_VERSION) for $PLATFORMS (push=$PUSH)"
echo ""

# --no-cache keeps the release compile honest; --target production selects the
# runtime stage only. Tags: semver + latest + git short SHA.
tags=(-t "$IMAGE:$VERSION" -t "$IMAGE:latest" -t "$IMAGE:$GIT_VERSION")
build=(docker buildx build --builder "$BUILDER" --platform "$PLATFORMS"
       --no-cache --target production "${tags[@]}"
       --build-arg RUST_BASE_IMAGE_VERSION="$RUST_BASE_IMAGE_VERSION"
       --build-arg DEBIAN_BASE_IMAGE_VERSION="$DEBIAN_BASE_IMAGE_VERSION"
       -f "$SCRIPT_DIR/Dockerfile" "$REPO_ROOT")

if [ "$PUSH" = "1" ]; then
    # --push uploads the per-platform images and the manifest list in one
    # step (the image never lands in the local docker store).
    "${build[@]}" --push
else
    # DRY RUN (default): a docker-container builder can't export a
    # multi-platform set into the local docker store, so the build is
    # exported as an OCI layout tarball — both platforms still compile and
    # the manifest is assembled, nothing leaves the machine.
    dest="${TMPDIR:-/tmp}/$NAME-$VERSION-multi-arch.tar"
    echo "exporting manifest-list tarball to $dest (no push)"
    "${build[@]}" --output "type=oci,dest=$dest"
fi
