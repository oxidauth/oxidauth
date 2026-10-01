# Production image for oxidauth-web (admin console).
#
# Build context is the REPO ROOT (the whole cargo workspace):
#
#   docker build --target production \
#     -f src/oxidauth/oxidauth-web/build/Dockerfile .
#
# Usually run through build/build.sh. The console is a pure-CSR wasm bundle,
# so the runtime stage is plain nginx serving the trunk dist — no server,
# no volumes. OXIDAUTH_API_URL / OXIDAUTH_CLIENT_KEY are compile-time
# (`option_env!` in src/state/mod.rs): they are ARGs here so the VALUES ARE
# BAKED INTO THE SERVED JS. Both are public-by-design browser inputs (any
# visitor can read them from the bundle); never bake a secret into this lane.

# ***********
# * Builder *
# ***********

ARG RUST_BASE_IMAGE_VERSION=v1.89.0
ARG NGINX_BASE_IMAGE_VERSION=1.27.3

FROM registry.vizerapp.cloud/pub/rust-base:${RUST_BASE_IMAGE_VERSION} AS builder

WORKDIR /home/rust

RUN apt update -y && apt install -y ca-certificates

RUN rustup target add wasm32-unknown-unknown

RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
    && cargo binstall --no-confirm trunk

COPY . .

# compile-time console config (see header)
ARG OXIDAUTH_API_URL=http://api.oxidauth.localhost
ARG OXIDAUTH_CLIENT_KEY
ENV OXIDAUTH_API_URL=${OXIDAUTH_API_URL} \
    OXIDAUTH_CLIENT_KEY=${OXIDAUTH_CLIENT_KEY}

RUN cargo build --release --target wasm32-unknown-unknown --package oxidauth-web \
    && cd src/oxidauth/oxidauth-web \
    && trunk build --release

# **************
# * Production *
# **************

FROM nginx:${NGINX_BASE_IMAGE_VERSION} AS production

COPY --from=builder /home/rust/src/oxidauth/oxidauth-web/dist /usr/share/nginx/html

RUN rm /etc/nginx/conf.d/default.conf

COPY src/oxidauth/oxidauth-web/build/nginx/nginx.conf /etc/nginx/nginx.conf
COPY src/oxidauth/oxidauth-web/build/nginx/default.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
