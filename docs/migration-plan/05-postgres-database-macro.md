# 05 — Postgres via `database!` macro

**Status**: `in-progress` (worker dispatched 2026-09-29)
**Depends on**: 04
**Risk**: medium — touches every query impl's pool access.

## Goal

`oxidauth-postgres` stops hand-rolling `Database` and adopts
`src/xlib/postgres`'s `database!` macro — gaining read/write pool separation,
`MIGRATIONS_ENABLED` gating, `PingTrait`, mock support, and test-facing
`MIGRATOR`, exactly like parkinglot's `admin-postgres`.

Current state: `oxidauth-postgres/src/lib.rs` defines
`Database { pool: PgPool }`, `from_env()` reading `DATABASE_URL` only, and
`migrate()` unconditionally running `sqlx::migrate!()`.

## Changes

1. **Add dep**: `oxidauth = xlib/postgres` in
   `src/oxidauth/oxidauth-postgres/Cargo.toml`
   (`postgres = { version = "0.1.0", path = "../../xlib/postgres" }`; the
   vendored crate's package name is `postgres` — if that collides with
   `sqlx`'s feature naming in manifests, alias with
   `postgres-xlib = { package = "postgres", path = … }`; keep package name
   `postgres` per template, use import alias in code).
2. **Replace `lib.rs` Database** with the macro call (template contract):
   ```rust
   const DATABASE_URL: &str = "DATABASE_URL";
   const READ_DATABASE_URL: &str = "READ_DATABASE_URL";
   const MIGRATIONS_ENABLED: &str = "MIGRATIONS_ENABLED";
   pub const MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

   database!(DATABASE_URL, READ_DATABASE_URL, MIGRATIONS_ENABLED, MIGRATOR);
   ```
   Passing `MIGRATOR` explicitly keeps the `./migrations` path pinned and
   exports the const for tests (`#[sqlx::test(MIGRATOR)]`), per the macro's
   doc examples. Keep `pub use` of `PgError/Ping/PingTrait/mock` (macro
   re-exports them; parkinglot's admin-postgres lib.rs does the same).
   Delete hand-written `Database`, `from_env`, `migrate`, `pool()`.
3. **Pool access sweep** in every `impl` module
   (`oxidauth-postgres/src/<entity>/<query>/mod.rs`):
   `self.pool.acquire()` / `&self.pool` → `self.write_pool()` /
   `self.read_pool()`. Heuristic mapping —
   `insert_*/update_*/delete_*/upsert_*` → write; `select_*` → read.
   ~60 files; mechanical. (Per-entity repository structs come in plan 08;
   keep `impl Service<&Req> for Database` shape here so this diff stays
   pool-mechanical.)
4. **Provider store** (plan 04's `provider/postgres.rs`):
   `Database::from_env()` now comes from the macro; keep `ping()` +
   `migrate()`; `provider.store(db.clone())` unchanged (macro Database is
   `Clone`).
5. **Migrations gating**: `MIGRATIONS_ENABLED=true` in dev compose (plan 10
   formalizes) and `.env`; production installs run migrations disabled per
   parkinglot (jobs/helm do it). Behavior change vs today's unconditional
   `migrate()` — call out in PR; missing env var is now a hard
   `PgError::MissingEnvVar` (fail-fast, template-sanctioned).
6. **Env vars**: `DATABASE_URL`, optional `READ_DATABASE_URL` (falls back to
   write URL inside `from_database_url`), `MIGRATIONS_ENABLED` — all already
   in plan 01's `example.env`.
7. **Tests**: convert `#[sqlx::test]` stubs to `#[sqlx::test(MIGRATOR)]` and
   delete the empty `_pool: PgPool {}` no-op tests (e.g.
   `select_user_by_id_query/tests/it_should_query_a_user_by_id_successfully`
   asserts nothing — remove rather than re-pin). Where unit tests need a DB
   stand-in, use `postgres::mock::MockPing` style doubles from the xlib mock
   module.
8. Keep the `.sql`-file pattern (`include_str!("./x.sql")` +
   `<query>.sql` per directory): this is template-blessed (xlib's own
   `ping/ping.sql`) — parkinglot's inline strings are a project choice, not a
   convention. Keep sqlx features as-is (`runtime-tokio`, `tls-rustls`,
   `macros`, `migrate`, `postgres`, `uuid`), bump sqlx 0.8.2 → 0.8.6 (full
   sweep in plan 14).

## Verification

- `cargo check -p oxidauth-postgres` green.
- `DATABASE_URL=*** READ_DATABASE_URL=$DATABASE_URL MIGRATIONS_ENABLED=false \
  cargo run --bin oxidauth-http` logs "migrations are not enabled" and boots;
  with `=true` it migrates (fresh DB via `bin/reset-db.sh`).
- `MIGRATIONS_ENABLED` unset → boot fails with `MissingEnvVar` (intended).
- `bin/hurl-tests.sh` full suite green (exercises read/write paths through
  the real server).
- `grep -rn 'self.pool' src/oxidauth/oxidauth-postgres/src | wc -l` → 0.

## PR note

changelog `<id>-postgres-database-macro`; Status → `done`.
