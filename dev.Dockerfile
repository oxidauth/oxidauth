ARG TARGETVERSION
ARG TARGETPLATFORM

FROM --platform=$TARGETPLATFORM registry.vizerapp.cloud/lib/rust-dev:$TARGETVERSION

ENV PATH=/root/.cargo/bin:$PATH

RUN cargo install watchexec-cli

WORKDIR /home/rust/src/oxidauth
