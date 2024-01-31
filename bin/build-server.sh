#!/bin/bash

cross build --target x86_64-unknown-linux-gnu --bin oxidauth-http --release

mkdir -p tmp

cp $CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/oxidauth-http tmp/oxidauth-http

docker buildx create --name oxidauth-builder --use

# VERSION=$(cat Cargo.toml | grep version | head -1 | cut -d'"' -f 2 | tr -d '\n')
VERSION=$(git rev-parse --short HEAD)
CROSS_CONTAINER_ENGINE_NO_BUILDKIT=1
PLATFORMS=linux/amd64
# PLATFORMS=linux/amd64,linux/arm64

IMAGE=registry.vizerapp.cloud/oxidauth/oxidauth-http:$VERSION
docker buildx build --platform=$PLATFORMS -t $IMAGE --push -f oxidauth-http/Dockerfile .

# rm -rf tmp
