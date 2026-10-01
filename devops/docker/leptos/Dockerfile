ARG TARGETVERSION=latest

FROM registry.vizerapp.cloud/pub/rust-dev:$TARGETVERSION

ARG TARGETPLATFORM

RUN cargo install trunk --git https://github.com/trunk-rs/trunk --tag v0.22.0-beta.2
RUN rustup target add wasm32-unknown-unknown

EXPOSE 80
