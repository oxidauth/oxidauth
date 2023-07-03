#!/bin/bash

# export required env vars
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
export CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc
export CXX_x86_64_unknown_linux_gnu=x86_64-linux-gnu-g++
export AR_x86_64_unknown_linux_gnu=x86_64-linux-gnu-ar

# cross compile
# RUSTFLAGS="-C link-args=-fstack-protector-all -lssp" cargo build --verbose --release --target x86_64-unknown-linux-gnu --bin oxidauth-server 
cargo build --verbose --release --target x86_64-unknown-linux-gnu --bin oxidauth-server 

cp $CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/oxidauth-server tmp/oxidauth-server 

VERSION=$(cat Cargo.toml | grep version | head -1 | cut -d'"' -f 2 | tr -d '\n')
IMAGE=registry.vizerapp.cloud/pub/oxidauth-server:$VERSION

docker buildx create --name oxidauth-test --use
docker buildx build --platform=linux/amd64 -t $IMAGE --push -f Dockerfile . 

rm tmp/oxidauth-server
