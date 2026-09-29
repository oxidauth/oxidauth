# 08 — Postgres: per-entity repository structs

**Status**: `reviewed` (walkthrough 2026-09-29: approved as written)
**Depends on**: 05
**Risk**: medium-large, fully mechanical. 60 query modules.

## Goal

Replace the unusual `impl Service<&Req> for Database` pattern (every query
trait is implemented **on the Database type itself**) with the template /
parkinglot pattern: **`Pg<Entity>Repository` structs holding `Database`,
implementing the entity's query traits, one file per entity.**

Template contract (parkinglot `src/admin/admin-postgres/src/property.rs`):
```rust
pub struct PgPropertyRepository { db: Database }
impl PgPropertyRepository { pub fn new(db: Database) -> Self { … } }
#[async_trait] impl FindPropertyByIdQuery for PgPropertyRepository { … self.db.read_pool() … }
```

## Changes

1. **Per-entity consolidation**: current layout is
   `<entity>/<query_name>/{mod.rs, query.sql}` (e.g.
   `users/select_user_by_id_query/`). **Keep the per-query dir + .sql files**
   (template blesses: xlib `ping/mod.rs` + `ping.sql`); only change *who
   implements*:
   - `oxidauth-postgres/src/users/mod.rs`: declare
     `pub struct PgUserRepository { db: Database }` + `new`.
   - each `<entity>/<query>/mod.rs`:
     `impl Service<&FindUserById> for Database` →
     `impl SelectUserByIdQuery for PgUserRepository` (the trait already exists
     in `oxidauth-repository/src/users/select_user_by_id_query.rs` with the
     right signature — the repo trait files stop being dead weight).
   - method name: repository traits already define `call(...)`-style methods —
     keep the trait's existing method name/signature; only the receiver type
     and pool accessor change (`self.write_pool()` →
     `self.db.write_pool()`).
   - free `async fn select_user_by_id_query(conn, …)` helpers: keep (they
     preserve testability + the .sql include), just call them from the new impl.
   Entities list (from `oxidauth-postgres/src/`): users, authorities,
   user_authorities, roles, role_role_grants, role_permission_grants,
   permissions, user_permission_grants, user_role_grants, refresh_tokens,
   public_keys, private_keys, totp_secrets, invitations, settings, auth/tree.
2. **Trait usage cleanup**: `oxidauth-repository` traits are the single trait
   system for data access; the `Service<&Xxx>` kernel trait is no longer
   implemented in postgres (it remains in kernel, deprecated, until 2.0 —
   handlers still call services via `Service` until plan 09).
3. **Type aliases**: `pub type PgUserRepo = PgUserRepository;`? No — no alias
   churn; provider (plan 04's `services.rs`) will construct
   `PgUserRepository::new(db.clone())` per repo and wrap in use cases (plan
   09). Interim (this plan): provider stores `Arc::new(PgXRepository)` under
   the repository trait object types where usecases already take repo traits;
   where a usecase takes `XService` backed by `Database` directly, adapt the
   usecase constructor minimally in this PR (`AuthenticateUseCase` takes 8
   repo-ish Arcs — its params get the Pg repos instead of `Database` clones).
   Keep changes compile-minimal; the services-side cleanup is plan 09.
4. Delete now-unused `impl` glue: any `Service<&X> for Database` blocks left
   unreferenced after the sweep must go (no dead code — `#![deny(dead_code)]`
   check).
5. `auth/tree` (recursive permission-tree query, uses `async-recursion`):
   becomes `PgAuthRepository`; keep recursion as-is.

## Verification

- `grep -rn 'impl.*for Database' src/oxidauth/oxidauth-postgres/src | wc -l` → 0.
- `cargo clippy --workspace -- -D warnings` green (expect unused-import noise
  from the sweep; clean it in the same PR).
- Full `bin/hurl-tests.sh` against fresh DB (`MIGRATIONS_ENABLED=true`) —
  exercises every endpoint end-to-end, i.e. every repo path.
- `cargo test -p oxidauth-postgres` (DB tests via `bin/database_test.sh` env)
  green.
- Spot-check one read path hits the read pool: set `READ_DATABASE_URL` to a
  *different* DB name; a users `GET` succeeds while a write fails on the
  missing schema (proves pool separation wired correctly).

## PR note

changelog `<id>-pg-repositories`; consider splitting PR per entity cluster
(users+authorities+grants / auth+jwt / rest) — same plan, sequential PRs,
Status flips on the last one.
