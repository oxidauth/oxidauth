#!/bin/bash

# export required env vars
# export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
# export CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc
# export CXX_x86_64_unknown_linux_gnu=x86_64-linux-gnu-g++
# export AR_x86_64_unknown_linux_gnu=x86_64-linux-gnu-ar

# cross compile
# export OPENSSL_INCLUDE_DIR=$(brew --prefix openssl)/include
# export OPENSSL_LIB_DIR=$(brew --prefix openssl)/lib

# cargo build --verbose --release --target x86_64-unknown-linux-gnu --bin oxidauth-server 
# RUSTFLAGS="-C link-args=-fstack-protector-all -lssp" cargo build --verbose --release --target x86_64-unknown-linux-gnu --bin oxidauth-server 

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
