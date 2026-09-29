- [90000005](https://www.pivotaltracker.com/story/show/90000005) - oxidauth-postgres adopts the xlib `database!` macro
    - `oxidauth-postgres/src/lib.rs` no longer hand-rolls `Database`/`from_env`/
      `migrate`/`ping`; it declares `pub const MIGRATOR: sqlx::migrate::Migrator
      = sqlx::migrate!("./migrations")` and calls
      `postgres::database!(DATABASE_URL, READ_DATABASE_URL, MIGRATIONS_ENABLED,
      MIGRATOR)`, gaining read/write pool separation, `MIGRATIONS_ENABLED`
      gating, `PingTrait` (pings both pools), `DatabaseBuilder`, mock support,
      and a test-facing `MIGRATOR` — the same shape as parkinglot's
      `admin-postgres`. The macro itself re-exports `PgError`, `Ping`,
      `PingTrait`, and `mock` at the crate root; `pub const MIGRATOR` is
      exported for `#[sqlx::test(MIGRATOR)]`
    - new path dep `postgres = { version = "0.1.0", path = "../../xlib/postgres" }`
      (package `postgres`); sqlx versions were already unified at 0.8.6 in the
      lock and nothing was bumped
    - pool-access sweep in every query impl: insert/update/delete/upsert and
      the transactional `insert_totp_secrets` use `self.write_pool()`
      (33 modules), `select_*` use `self.read_pool()` (29 modules);
      `self.pool` no longer appears anywhere in the crate
    - `oxidauth-http` `provider/postgres.rs`, `health.rs`, `live.rs` import
      `PingTrait` for `ping()` (it is now a trait method); `Database::from_env`
      keeps its call sites, now returning `Result<Database, PgError>`;
      `provider.store(db.clone())` unchanged (macro `Database` is `Clone`)
    - BEHAVIOR CHANGE: `migrate()` is now gated on `MIGRATIONS_ENABLED`. A
      missing `MIGRATIONS_ENABLED` env var is a hard `PgError::MissingEnvVar`
      fail-fast at boot (previously `migrate()` ran `sqlx::migrate!()`
      unconditionally). Every deployment MUST set it: `MIGRATIONS_ENABLED=true`
      where the app runs migrations (dev), `MIGRATIONS_ENABLED=false` where
      jobs/helm do (prod). `READ_DATABASE_URL` is optional and falls back to
      `DATABASE_URL`
    - removed 47 no-op `mod tests` blocks (50 stub fns: 47 empty
      `_pool: PgPool {}` bodies, 3 fully-commented bodies; one module held only
      a commented-out test) — they asserted nothing, so they were deleted
      rather than re-pinned; no real `#[sqlx::test]` bodies existed to convert
      to `#[sqlx::test(MIGRATOR)]`, and `mock::PingMock` remains available for
      unit tests that need a DB stand-in
