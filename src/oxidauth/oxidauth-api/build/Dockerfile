# Production image for oxidauth-api (migration plan 12).
#
# Built by build/build.sh with the REPO ROOT as context (`COPY . .` takes the
# whole cargo workspace; `.dockerignore` trims target/, .env, secrets):
#
#   docker buildx build --platform linux/amd64,linux/arm64 \
#     -f src/oxidauth/oxidauth-api/build/Dockerfile .
#
# The server binds $PORT (default 80 — oxidauth-api/src/main.rs), so EXPOSE 80
# and the Helm chart's api.containerPort: 80 agree; the chart also injects
# PORT explicitly.

# ***********
# * Builder *
# ***********

ARG RUST_BASE_IMAGE_VERSION=v1.89.0
ARG DEBIAN_BASE_IMAGE_VERSION=12.12

FROM registry.vizerapp.cloud/pub/rust-base:${RUST_BASE_IMAGE_VERSION} AS builder

WORKDIR /home/rust

RUN apt update -y && apt install -y ca-certificates

COPY . .

RUN cargo build --release --bin oxidauth-api

# **************
# * Production *
# **************

FROM debian:${DEBIAN_BASE_IMAGE_VERSION} AS production

RUN apt update -y && apt install -y ca-certificates

COPY --from=builder /home/rust/target/release/oxidauth-api /bin/oxidauth-api
# sqlx `migrate!` embeds the migrations into the binary (compile-time), but
# they are copied alongside too — parkinglot parity for ad-hoc `sqlx migrate`
# ops against a running container. Nothing in the server reads MIGRATIONS_PATH.
COPY --from=builder /home/rust/src/oxidauth/oxidauth-postgres/migrations /etc/oxidauth/oxidauth-api/migrations

EXPOSE 80

CMD ["/bin/oxidauth-api"]
