- [90000013](https://www.pivotaltracker.com/story/show/90000013) - seedz:
  local-dev fixture seeder (migration plan 13).
    - `src/seedz/` rewritten wholesale (the `oxidauth-seed` stub's `add()`
      body dies here): package `seedz` with `[[bin]] name = "seedz"`.
      `lib.rs` is the template's project-level seeder API — `Seeder` trait
      (`seed(&self, write: &PgPool, read: &PgPool)`) + `SeedRunner`
      (add_seeder/run, sequential) — with `BoxedError` re-exported from
      `oxidauth_kernel::error` instead of the template's local definition.
    - `main.rs`: **dev-only guard first** — unless `ENVIRONMENT=local`,
      prints "seedz is for local development; refusing to run against
      ENVIRONMENT=<x>" and exits 1 before telemetry, before any DB handle
      (verified against production: exit 1, zero rows written). Then
      telemetry boot (xlib crate, like the api), `oxidauth_postgres::
      Database::from_env()` (DATABASE_URL + READ_DATABASE_URL),
      `database.migrate()` (plan-05 API: hard-fails on missing
      `MIGRATIONS_ENABLED`, skips unless `"true"`), then the fixtures
      seeder via SeedRunner. No hand-rolled pools; `MIGRATOR` rides the
      Database. **Server boot untouched**: `oxidauth-api/src/main.rs` and
      `oxidauth-services/src/bootstrap/` carry no changes (bootstrap stays
      the provisioning path; seedz never runs from the server).
    - `fixtures.rs` — idempotent dev data via find-then-insert on each
      table's natural key (name / username / realm+resource+action / PK);
      every helper RESOLVES the existing row's live id and threads it into
      dependent grant inserts, so pre-existing rows — earlier seed runs OR
      rows created by hand/API under the same names (random UUIDs) — are
      reused, never duplicated and never an FK abort; deterministic
      `f0000000-0000-4000-8000-0000000000XX` UUIDs (parkinglot convention)
      on fresh inserts keep re-runs convergent. Inventory: authority
      `local-dev` (username_password, enabled, settings JSON mirroring
      what bootstrap's `AuthoritySettings` serializes, fixed dev
      `password_salt`, pinned client_key `f0…02` for probes), users
      `seedz:viewer` / `seedz:editor` / `seedz:auditor` (kind human)
      attached to `local-dev`, covering the three grant combinations
      (role-only; role + direct `demo:reports:export`; direct-only
      `demo:audit:read`), roles `seedz:viewer` / `seedz:editor` over demo
      `realm:resource:action` permissions, and setting `seedz:sample`.
      No signing keys, no `oxidauth:admin`, no default authority —
      bootstrap owns those; no credentials either (passwords are
      argon2+pepper at registration, this seeder never fakes them).
    - Stack compose: one-shot `seedz` service, `profiles: [seed]`,
      `depends_on: postgres: {condition: service_healthy}` (matches the
      api; `PgPool::connect` is eager with no retry, so even a one-shot
      must not race postgres warmup), in-cluster DB URLs,
      `MIGRATIONS_ENABLED="true"`, `ENVIRONMENT=local`, shared-vol cargo
      cache; run with `docker compose --profile seed run --rm seedz`
      (repo root). Plan deviation: image is the locally-built
      `oxidauth-dev:local` (same build block as `oxidauth-api`, compose
      dedupes) instead of the raw `rust-dev` registry tag — plan 10
      standardized this stack's dev image on the local build; plus
      `CARGO_TARGET_DIR` (the plan snippet mounted shared-vol but omitted
      the var; without it the mount is pointless).
    - Verified fresh-DB end to end: reset-db → boot (bootstrap provisions,
      setting written) → seedz ×2 (second run: all-skip/reuse, row counts
      identical across users/roles/authorities/settings/permissions/
      grants) → api restart logs "bootstrap already completed",
      authorities stay 2 (no duplicate) → hurl suite green (1+17+17) on
      the seeded DB. Hand-created-collision run (role/permission made via
      psql with random UUIDs before first seed): seedz succeeds, grants
      point at the live rows, counts stable. `cargo check -p seedz` + fmt
      green.
