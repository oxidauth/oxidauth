#!/bin/bash

cross build --target x86_64-unknown-linux-gnu --release

cp $CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/oxidauth-server tmp/oxidauth-server

docker buildx create --name oxidauth-builder --use

VERSION=$(cat Cargo.toml | grep version | head -1 | cut -d'"' -f 2 | tr -d '\n')
CROSS_CONTAINER_ENGINE_NO_BUILDKIT=1
PLATFORMS=linux/amd64,linux/arm64

IMAGE=registry.vizerapp.cloud/pub/oxidauth-server:$VERSION
docker buildx build --platform=$PLATFORMS -t $IMAGE --push -f Dockerfile .

IMAGE=registry.vizerapp.cloud/prism/oxidauth-server:$VERSION
docker buildx build --platform=$PLATFORMS -t $IMAGE --push -f Dockerfile .

rm tmp/oxidauth-server
