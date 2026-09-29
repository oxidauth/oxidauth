# 10 — Compose + dev environment

**Status**: `reviewed` (walkthrough 2026-09-29: approved as written — dev.Dockerfile retired for registry rust-dev image)
**Depends on**: 05, 06
**Risk**: low-medium; developer workflow changes (paths + commands).

## Goal

Topology flip to template shape: the **root** compose becomes an `include:`
manifest; the **stack** owns its dev compose; the hand-rolled `dev.Dockerfile`
dies in favor of the shared `rust-dev` registry image.

Reference: `/tmp/migcmp/tmpdemo/src/tmpstack/docker-compose.yml` (generated)
and parkinglot `src/admin/docker-compose.yml` (wired + proxied variant).

## Changes

1. **New `src/oxidauth/docker-compose.yml`** (stack dev compose):
   ```yaml
   services:
     postgres:
       image: postgres:16
       platform: $DOCKER_PLATFORM
       environment:
         POSTGRES_USER: postgres
         POSTGRES_PASSWORD: ***
         POSTGRES_DB: oxidauth
       ports: [ "5434:5432" ]          # keep 5434 host port (dev-box convention)
       healthcheck: { test: ["CMD-SHELL","/bin/sh","pg_isready -U postgres"], interval: 5s, timeout: 5s, retries: 5 }
       volumes: [ postgres_data:/var/lib/postgresql/data ]

     oxidauth-api:
       working_dir: /home/rust/src/oxidauth
       command: /bin/bash -c 'watchexec -c -e rs -r cargo run --bin oxidauth-api'
       image: registry.vizerapp.cloud/lib/rust-dev:$RUST_DEV_IMAGE_VERSION
       platform: $DOCKER_PLATFORM
       stdin_open: true
       tty: true
       depends_on:
         postgres: { condition: service_healthy }
       environment:
         CARGO_TARGET_DIR: /home/rust/shared_target
         DATABASE_URL: postgresql://postgres:***@postgres:5432/oxidauth
         MIGRATIONS_ENABLED: "true"
         ENVIRONMENT: local
         RUST_LOG: ${RUST_LOG:-INFO}
         VIRTUAL_HOST: api.oxidauth.localhost
       env_file: [ ../../.env ]
       ports: [ "80" ]
       volumes:
         - "shared-vol:/home/rust/shared_target"
         - ../../:/home/rust/src/oxidauth:cached
       networks:
         default: { aliases: [ api.oxidauth.localhost ] }

   volumes:
     postgres_data:
   ```
   (`shared-vol:` is declared by the **root** file; include semantics share
   it. `VIRTUAL_HOST` alias kept from the old file for local proxy routing.)
2. **Root `docker-compose.yml`** shrinks to:
   ```yaml
   include:
     - ./src/oxidauth/docker-compose.yml
   volumes:
     shared-vol:
   ```
3. **Delete `dev.Dockerfile`** (compose now runs the published
   `registry.vizerapp.cloud/lib/rust-dev:$RUST_DEV_IMAGE_VERSION` image, like
   the generated stack; the old base image rev v1.76.0 was ancient anyway —
   plan 01 sets v1.98.0). If local `sqlx-cli`/`hurl` tooling isn't in
   `rust-dev`, add a `devops/docker/oxidauth-dev/Dockerfile` FROM rust-dev
   that `cargo install`s exactly the missing tools — only if actually needed.
4. **Postgres provisioning alignment**: old flow relied on `POSTGRES_*`
   env_file + postgres image's implicit single-db init. Adopt
   `devops/postgres/init.sh` (plan 01) mounted at
   `/docker-entrypoint-initdb.d/` with `SERVICES=oxidauth` so DB/user creation
   is scriptable for multi-service local stacks later (parkinglot runs
   oxidauth + admin DBs side by side — this is the interop path).
5. **Env contract**: api container reads `.env` at project root
   (`../../.env` relative to stack compose) — matches generated stack. Local
   `cargo run` outside docker still uses `bin/cargo-watch.sh`-style
   `DATABASE_URL=postgres://oxidauth:***@127.0.0.1:5434/oxidauth` (via
   `.env`, plan 11 refreshes the script).
6. Keep `bin/reset-db.sh` (sqlx database drop/create/migrate) but read
   `DATABASE_URL` from `.env` instead of hardcoding.

## Verification

- `docker compose config` at root resolves the include (no dangling refs,
  shared-vol defined once).
- `docker compose up -d` from root: postgres healthy, api watchexec boots
  ("starting oxidauth-api" bunyan line), `curl
  http://localhost:$(docker compose port oxidauth-api 80)/api/v1/__meta/healthcheck`
  → 200 healthy:true.
- Second run reuses `shared-vol` cache (target dir persists across `up/down`).
- `bin/reset-db.sh` green.

## PR note

changelog `<id>-compose-stack-layout`; Status → `done`.
