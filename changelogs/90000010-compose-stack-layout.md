- [90000010](https://www.pivotaltracker.com/story/show/90000010) - compose
  topology flip: the **stack** owns its dev compose
  (`src/oxidauth/docker-compose.yml`); the root `docker-compose.yml` shrinks to
  an `include:` of it plus the shared `shared-vol` declaration (include
  semantics share it — defined exactly once).
    - `dev.Dockerfile` deleted. The api service now builds
      `devops/docker/oxidauth-dev/Dockerfile` (tagged `oxidauth-dev:local`)
      FROM `registry.vizerapp.cloud/lib/rust-dev:$RUST_DEV_IMAGE_VERSION` —
      the hand-rolled dev image only added `watchexec-cli`, which is already
      baked into rust-dev; the overlay adds exactly the two missing dev CLIs:
      `sqlx-cli` (pinned to the workspace sqlx 0.8.6, `--no-default-features
      --features postgres,rustls` — 0.8's migrator is built-in, no `migrate`
      feature) for `bin/reset-db.sh`, and `hurl`
      (official GitHub release binary, arch-selected via `TARGETARCH`, 8.0.1 —
      upstream dropped static-musl assets, gnu builds ship instead) for the
      plan-11 hurl suite.
    - Postgres provisioning is scriptable via
      `devops/postgres/init.sh` mounted at
      `/docker-entrypoint-initdb.d/init.sh` (ro) with `SERVICES=oxidauth`:
      the container boots with `POSTGRES_DB` = `postgres` (the superuser
      role/db the image creates) and init.sh creates database + role
      `oxidauth` (password `oxidauth`) — the role is **owner of its database
      with `CREATEDB`**, which is what makes `bin/reset-db.sh`
      (sqlx drop/create) work with plain `.env` credentials; the old implicit
      flow got this because docker-postgres makes `POSTGRES_USER` a
      superuser. Multi-service local stacks (oxidauth + admin side by side)
      therefore get one provisioning path. The init script only runs on an
      empty data volume — switching to it needs `docker compose down -v`
      once.
    - Conventions kept: postgres host port **5434** (`.env` URLs point at
      `127.0.0.1:5434`), cargo target dir on named volume `shared-vol`
      (`CARGO_TARGET_DIR=/home/rust/shared_target`, survives up/down), repo
      mounted at `/home/rust/src/oxidauth`, api on ephemeral host port 80
      (`docker compose port oxidauth-api 80`) with the
      `api.oxidauth.localhost` network alias + `VIRTUAL_HOST` for local proxy
      routing. `env_file: ../../.env` — both `DATABASE_URL` and
      `READ_DATABASE_URL` (…@postgres:5432/oxidauth) are service-env
      overrides of the host URLs in `.env`: with the plan-05 read/write pool
      split, leaving the read URL pointing at `127.0.0.1:5434` makes the
      in-container read pool hang and the boot dies with `PoolTimedOut`.
    - `DOCKER_PLATFORM` (`.env`/`example.env`) now defaults to empty =
      host-native platform (compose omits `platform:`), replacing the pinned
      `linux/amd64`; pin it only to force cross-arch emulation.
    - `bin/reset-db.sh` no longer hardcodes `DATABASE_URL=…@127.0.0.1:5432`;
      it sources the repo-root `.env` (`set -a; source …; set +a`) and keeps
      the sqlx drop/create/migrate flow against the mapped 5434 port. Stop
      `oxidauth-api` first (`docker compose stop oxidauth-api`) — sqlx
      cannot drop the database while the api pool holds connections.
