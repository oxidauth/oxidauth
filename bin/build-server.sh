#!/bin/bash

cross build --target x86_64-unknown-linux-gnu --bin oxidauth-http --release
cross build --target aarch64-unknown-linux-gnu --bin oxidauth-http --release

mkdir -p ./tmp/linux/amd64
mkdir -p ./tmp/linux/arm64

cp $CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/oxidauth-http tmp/linux/amd64
cp $CARGO_TARGET_DIR/aarch64-unknown-linux-gnu/release/oxidauth-http tmp/linux/arm64

docker buildx create --name oxidauth-builder --use

SEMVER_VERSION=$(cat oxidauth-http/Cargo.toml | grep version | head -1 | cut -d'"' -f 2 | tr -d '\n')
GIT_VERSION=$(git rev-parse --short HEAD)
CROSS_CONTAINER_ENGINE_NO_BUILDKIT=1
# PLATFORMS=linux/amd64
PLATFORMS=linux/amd64,linux/arm64

IMAGE=registry.vizerapp.cloud/oxidauth/oxidauth-http

docker buildx build \
  --platform=$PLATFORMS \
  -t $IMAGE:$SEMVER_VERSION \
  -t $IMAGE:$GIT_VERSION \
  --push \
  -f oxidauth-http/Dockerfile .
