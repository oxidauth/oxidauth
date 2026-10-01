#!/bin/bash
set -euo pipefail

# Build (and optionally push) the multi-arch production image for
# oxidauth-web. Run directly or via the repo-root bin/build.sh, which
# exports REGISTRY / RUST_BASE_IMAGE_VERSION / NGINX_BASE_IMAGE_VERSION.
#
#   OXIDAUTH_CLIENT_KEY=<uuid> PUSH=1 ./build.sh
#   OXIDAUTH_CLIENT_KEY=<uuid> ./build.sh   # DEFAULT: dry run, OCI tarball
#
# Same push-gate posture as oxidauth-api/build/build.sh (plan 12): the
# registry push is an explicit human gate.
#
# OXIDAUTH_CLIENT_KEY is REQUIRED (it is baked into the bundle — public by
# design, but a build without it ships a console that cannot authenticate).
# OXIDAUTH_API_URL defaults to the dev-proxy api host.

NAME=oxidauth-web

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"

VERSION="$(bash "$REPO_ROOT/bin/crate_version.sh" "$NAME")"
GIT_VERSION="$(git -C "$REPO_ROOT" rev-parse --short HEAD)"
REGISTRY="${REGISTRY:-registry.vizerapp.cloud/oxidauth}"
IMAGE="$REGISTRY/$NAME"
PLATFORMS="${DOCKER_PLATFORMS:-linux/amd64,linux/arm64}"
BUILDER=oxidauth-builder
PUSH="${PUSH:-0}"
RUST_BASE_IMAGE_VERSION="${RUST_BASE_IMAGE_VERSION:-v1.89.0}"
NGINX_BASE_IMAGE_VERSION="${NGINX_BASE_IMAGE_VERSION:-1.27.3}"
OXIDAUTH_API_URL="${OXIDAUTH_API_URL:-http://api.oxidauth.localhost}"
OXIDAUTH_CLIENT_KEY="${OXIDAUTH_CLIENT_KEY:?set OXIDAUTH_CLIENT_KEY (the console authority uuid)}"

export DOCKER_BUILDKIT=1

if ! docker buildx inspect "$BUILDER" > /dev/null 2>&1; then
  echo "creating multi-platform builder $BUILDER"
  docker buildx create --name "$BUILDER" --driver docker-container
  docker run --privileged --rm tonistiigi/binfmt --install all
fi

echo "building $NAME $VERSION ($GIT_VERSION) for $PLATFORMS (push=$PUSH)"
echo "  api url: $OXIDAUTH_API_URL"
echo ""

tags=(-t "$IMAGE:$VERSION" -t "$IMAGE:latest" -t "$IMAGE:$GIT_VERSION")
build=(docker buildx build --builder "$BUILDER" --platform "$PLATFORMS"
       --no-cache --target production "${tags[@]}"
       --build-arg RUST_BASE_IMAGE_VERSION="$RUST_BASE_IMAGE_VERSION"
       --build-arg NGINX_BASE_IMAGE_VERSION="$NGINX_BASE_IMAGE_VERSION"
       --build-arg OXIDAUTH_API_URL="$OXIDAUTH_API_URL"
       --build-arg OXIDAUTH_CLIENT_KEY="$OXIDAUTH_CLIENT_KEY"
       -f "$SCRIPT_DIR/Dockerfile" "$REPO_ROOT")

if [ "$PUSH" = "1" ]; then
    "${build[@]}" --push
else
    dest="${TMPDIR:-/tmp}/$NAME-$VERSION-multi-arch.tar"
    echo "exporting manifest-list tarball to $dest (no push)"
    "${build[@]}" --output "type=oci,dest=$dest"
fi
