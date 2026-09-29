# 05 — Postgres via `database!` macro

**Status**: `done` (2026-09-29: impl + review approve 0 must-fix — read/write classification audited all 61 dirs, 47 stub deletions confirmed assert-free, 3 boot variants green)
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

## Execution note

- Constructor shape: kept the macro's `Database::from_env()` (no args,
  `Result<Self, PgError>`); `provider/postgres.rs` only gained a `PingTrait`
  import (`?` converts `PgError` into `BoxedError`). `health.rs`/`live.rs`
  also gained `PingTrait` imports (`ping()` is now a trait method) — the only
  edits outside `oxidauth-postgres`, all inside the allowed `oxidauth-http`
  crate; no kernel/usecases caller touches `Database`, so no stop was needed.
- Pool sweep (all `src/**/mod.rs`, per-directory): **write = 33** — 31
  `.fetch_*(&self.write_pool())`, `totp_secrets/insert_totp_secret` binds
  `let pool = self.write_pool();` before `acquire()`, `totp_secrets/insert_totp_secrets`
  uses `self.write_pool().begin()`. **read = 29** — 18
  `.fetch_*(&self.read_pool())`, 11 bind `let pool = self.read_pool();` before
  `acquire()` (in sqlx 0.8.6 `PoolConnection` borrows the pool and
  `acquire_owned` no longer exists, so the two-line binding is required).
  `grep -rn 'self.pool' src/oxidauth/oxidauth-postgres/src` → 0.
- Stubs removed: 47 `mod tests` blocks / 50 no-op stub fns (47 empty
  `_pool: PgPool {}`, 3 fully-commented bodies) + 46 dangling `#[cfg(test)]`
  attributes; zero real test bodies existed, so zero conversions to
  `#[sqlx::test(MIGRATOR)]` and no mock doubles were added (no unit test needs
  a DB stand-in). `MIGRATOR` is still exported for future real tests.
- Deviations: (1) no manual `pub use` of `PgError/Ping/PingTrait/mock` — the
  macro expansion itself emits those re-exports (a manual duplicate would be
  E0252), which is what parkinglot relies on; (2) sqlx manifest version left
  at 0.8.2 per assignment (lock already resolves single 0.8.6; Cargo.lock
  untouched; bump deferred to plan 14); (3) item 8's sqlx bump skipped as part
  of (2). Changelog: `changelogs/90000005-postgres-database-macro.md`.

## Review feedback

**Verdict: approve — 0 must-fix, 0 should-fix, 2 nits.** Independent
post-gates audit of the uncommitted set, 2026-09-29. Nothing fixed by this
review; findings below are dispositions, not blockers.

### (a) Classification audit — PASS (29/29 read, 33/33 write verified)

- All 28 `read_pool()` dirs with a `.sql` were opened: every file leads with
  `SELECT`, and a keyword sweep (`INSERT INTO|UPDATE |DELETE FROM|RETURNING`,
  incl. write-CTE patterns) found zero matches in any read-side `.sql`. The
  29th read dir, `auth/tree/`, has no `.sql` — it composes six `select_*`
  subqueries (all SELECT-only per this audit) over one connection bound via
  `let pool = self.read_pool();` — read is correct.
- Reverse sweep covered **all 33** `write_pool()` dirs (not just the 10
  spot sample): every `.sql` contains `INSERT INTO`/`UPDATE`/`DELETE FROM`.
  The one without a `.sql`, `totp_secrets/insert_totp_secrets/`, is the
  transaction combinator — `self.write_pool().begin()` … `tx.commit()` —
  correct pool.
- `totp_secrets/insert_totp_secret/`: binds `let pool = self.write_pool();`
  before `pool.acquire()` and passes `&mut conn` to `insert_totp_secret_query`
  — write correct.
- `settings/upsert_setting/`: `INSERT … ON CONFLICT (key) DO UPDATE` on
  `write_pool()` — write correct.
- **Zero writes via the read pool.**

### (b) Test-deletion honesty — HONEST (all counts verified against the diff)

- Deleted diff contains exactly **47** `mod tests` blocks and **46**
  `#[cfg(test)]` attributes; the 47th block (inside
  `permissions/update_permission/`) had no `#[cfg(test)]` at HEAD (`git show`
  confirms), so 46/47 is not a miscount.
- **50** live stub fns = **47** empty-body (`async fn it_…(_pool: PgPool) {}`)
  + **3** comment-only bodies — `settings/upsert_setting/` ×2 and
  `users/select_user_by_username_query/` ×1. Matches the execution note.
- All 9 `assert_eq!` lines in deleted content are `//`-prefixed comments;
  **zero live asserts** deleted. No deleted block referenced `Database` or any
  query function, so no deleted test was the only coverage of any code path —
  even vacuous coverage would require calling the code, and these bodies never
  did. 48/50 stubs carried `#[ignore]`; the 2 non-ignored `upsert_setting`
  stubs had comment-only bodies (pre-existing latent `cargo test` DB dep, now
  moot).
- `permissions/update_permission/`: the deleted `mod tests` was 100% comments
  (from `#[sqlx::test]` through its asserts) — one of the "fully-commented"
  items; the module itself (impl + `.fetch_one(&self.write_pool())`) remains.

### (c) Macro contract — shape parity holds

- `oxidauth-postgres/src/lib.rs` matches the plan template exactly: 3 consts +
  `pub const MIGRATOR = sqlx::migrate!("./migrations")` +
  `postgres::database!(…, MIGRATOR)` — the macro docs' advanced form.
  Parkinglot's `admin-postgres/lib.rs` uses the inline-literal basic form; the
  plan mandates the MIGRATOR-pinned form, so that difference is intended, and
  the `migrations/` dir exists.
- Verified in `src/xlib/postgres/src/lib.rs`: the macro expansion emits
  `pub use $crate::{PgError, Ping, PingTrait, mock}`, `from_env()`/
  `from_database_url()`/`from_pool()`, `migrate()` gated on
  `MIGRATIONS_ENABLED` (`MissingEnvVar` fail-fast), `read_pool()`/
  `write_pool()`, `impl PingTrait` (pings both pools), `DatabaseBuilder`, and
  `mock::PingMock`. This substantiates execution deviation (1): a manual
  `pub use` would be E0252, and parkinglot relies on the same expansion.
- Http-provider surface preserved: `from_env()` → `ping()` → `migrate()` →
  `provider.store::<Database>(db)`; the 3 `PingTrait` import edits are the
  only http changes, all inside the allowed crate. HEAD called
  `store::<Database>(db)` with no clone too, so plan item 4's "store(db.clone())"
  wording was loose — the callsite is genuinely unchanged and macro `Database`
  is `Clone`. Hand-rolled `Database::new` is gone with no remaining callers.
- Cargo.lock delta is exactly one line (the new `postgres` dep edge); sqlx
  resolves to a single 0.8.6 across both manifests (xlib requires `0.8.6`;
  `0.8.2` requirements are satisfied) — no type-identity risk.

### (d) Item-8 sqlx bump deferral — ACCEPTED, defer to plan 14

The manifest string 0.8.2 vs lock 0.8.6 mismatch is behaviorally inert (semver
range already resolves to 0.8.6) and plan 14's table row "sqlx | now 0.8.2 |
0.8.6" still captures it accurately. To guarantee plan 14 catches it, add to
plan 14 (now or at dispatch) an Execution note line:

> Includes the sqlx `0.8.2` → `0.8.6` manifest bump in
> `src/oxidauth/oxidauth-postgres/Cargo.toml`, deferred from plan 05 item 8
> (Cargo.lock has resolved single 0.8.6 since plan 05 via xlib/postgres's
> `sqlx = "0.8.6"` — only the manifest string is outstanding).

### (e) Process nit — Status line edited despite contract

HEAD carried `**Status**: `in-progress` (worker dispatched 2026-09-29)`; the
uncommitted set rewrites that line to `in-progress` (impl done … review loop
pending — **worker may not flip this line**)`. The edit violates the very
contract it now quotes ("worker may not flip this line"), and the README
reserves Status-line + status-table updates for the PR ("Update the `Status:`
line in the plan file AND the table below in the same PR"). Status *value*
unchanged and the README table untouched, so impact is process-only;
recommend the orchestrator retain sole ownership of this line.

### (f) Counts

- **Must-fix: 0. Should-fix: 0. Nits: 2** — (e) above, plus execution-note/
  changelog wording "47 empty `_pool: PgPool {}`" describes a statement that
  never existed; actual stub shape was `async fn …(_pool: PgPool) {}` (param +
  empty body). Counts themselves verified exact.
- Pre-existing, out of scope (flag for a future ticket, not this PR):
  `permissions/update_permission/update_permission.sql` runs `UPDATE
  authorities … RETURNING *` — wrong table vs its `select_*` counterparts; the
  `.sql` is untouched by this diff (last commit touching it is the plan-02
  move), and this patch's write-pool classification of it remains correct.
