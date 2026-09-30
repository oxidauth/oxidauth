# Dev image: shared rust-dev registry image + the two CLI tools the local dev
# flow needs (sqlx-cli for bin/reset-db.sh, hurl for bin/hurl.sh).
# Retires the old hand-rolled dev.Dockerfile (which only added watchexec —
# already baked into rust-dev).
ARG RUST_DEV_IMAGE_VERSION=v1.98.0
FROM registry.vizerapp.cloud/lib/rust-dev:${RUST_DEV_IMAGE_VERSION}

ARG TARGETARCH
# keep in lockstep with the workspace sqlx version (Cargo.lock)
ARG SQLX_CLI_VERSION=0.8.6
ARG HURL_VERSION=8.0.1
# 0.8 has no `migrate` feature (migrator is built-in); rustls avoids native-tls
RUN apt-get update \
    && apt-get install -y --no-install-recommends libxml2 \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install sqlx-cli --version ${SQLX_CLI_VERSION} --no-default-features --features postgres,rustls --locked \
    && case "${TARGETARCH}" in \
         amd64) HURL_ARCH=x86_64-unknown-linux-gnu ;; \
         arm64) HURL_ARCH=aarch64-unknown-linux-gnu ;; \
         *) echo "unsupported TARGETARCH: ${TARGETARCH}" && exit 1 ;; \
       esac \
    && curl -fsSL "https://github.com/Orange-OpenSource/hurl/releases/download/${HURL_VERSION}/hurl-${HURL_VERSION}-${HURL_ARCH}.tar.gz" \
       | tar xz -C /usr/local/bin --strip-components=2 "hurl-${HURL_VERSION}-${HURL_ARCH}/bin/hurl" \
    && hurl --version && sqlx --version
