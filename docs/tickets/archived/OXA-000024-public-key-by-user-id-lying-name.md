# OXA-000024 — Rename `select_public_key_by_user_id`: module does a primary-key lookup

**Original ID:** DATA-11 · **Severity:** P3 · **Type:** note · **Status:** implemented (2026-09-29)
**Review tier:** Tier 2 (mechanical 1-3 line code fix) — ranked #13 of 27 · reviewed 2026-09-29 · SCHEDULED (land early)


## Locations

All verified by reading the code; paths relative to `src/oxidauth/` unless noted.

- **The lying name (whole defect):** directory `oxidauth-postgres/src/public_keys/select_public_key_by_user_id/` and its declaration `oxidauth-postgres/src/public_keys/mod.rs:4` — `pub mod select_public_key_by_user_id;`. These two spots are the *only* occurrences of the string `select_public_key_by_user_id` in compiled code (workspace `rg`, `target/` excluded; remaining hits are `BUGS_AND_NOTES.md:43` and the `missing-tests.md` ledger at `:140,:473`).
- **The SQL inside it:** `oxidauth-postgres/src/public_keys/select_public_key_by_user_id/select_public_key_by_id.sql:1-7` — `SELECT id, public_key, created_at, updated_at FROM public_keys WHERE id = $1`. Note the SQL *file* is already honestly named `..._by_id.sql`; the include site is `mod.rs:16-18`, and the tracing span is already `select_public_key_by_id_query` (`mod.rs:11`).
- **Implementation:** `oxidauth-postgres/src/public_keys/select_public_key_by_user_id/mod.rs:7-26` — `impl Service<&'a FindPublicKeyById> for PgPublicKeyRepository`, binds `public_key_id: Uuid`, `fetch_one` on the read pool, maps through the sanitized row `PgPublicSanitizedKey` (`oxidauth-postgres/src/public_keys/mod.rs:45-51`) → `PublicKey`.
- **Schema:** `oxidauth-postgres/migrations/20221021180809_create_public_keys.sql` — `public_keys (id UUID PK, public_key BYTEA, private_key BYTEA, created_at, updated_at)`. No `user_id` column. It is the *only* migration ever touching `public_keys` (rg across `migrations/`), so the table has never had a user link.
- **Repository trait — already correct:** `oxidauth-repository/src/public_keys/select_public_key_by_id.rs` declares `SelectPublicKeyByIdQuery` (`:5-8`) over `Service<&FindPublicKeyById, Response = PublicKey, Error = BoxedError>` with a blanket impl (`:10-13`). There is **no** `select_public_key_by_user_id` symbol in oxidauth-repository or oxidauth-kernel; the query struct is `oxidauth_kernel::public_keys::find_public_key_by_id::FindPublicKeyById { public_key_id: Uuid }` (`oxidauth-kernel/src/public_keys/find_public_key_by_id.rs:14-17`).
- **The `BUG(pinned)` marker the register cites:** `select_public_key_by_user_id/mod.rs:41-43`, a doc comment above `it_should_find_the_sanitized_key_by_primary_id` (`:44-63`) stating exactly this misnomer.
- **Honest `by_user_id` siblings (naming convention in active use):** `user_role_grants/select_user_role_grants_by_user_id/`, `user_authorities/select_user_authorities_by_user_id/`, `refresh_tokens/delete_refresh_token_by_user_id/` (DATA-10 — registered separately, and its name is *honest*: those tables do have `user_id` per `migrations/20221020012659_create_user_role_grants.sql:2`, `20221020204155_create_user_authorities.sql:2`, `20221027183328_create_refresh_tokens.sql:3`, `20240416170202_create_totp_secrets.sql:3`, `20240619.../user_permission_grants` equivalents).

## Problem

`oxidauth-postgres`'s public_keys module tree advertises a `select_public_key_by_user_id` query that does not exist as a query: its SQL selects by the primary key `id`, and the `public_keys` table has no `user_id` column to select by. A reader or caller who trusts the module name expects "give me the keys belonging to user X" and gets "give me the one row whose `id` equals X" — with a `RowNotFound` error (`mod.rs:65-78` pins that) for any UUID that isn't a key id, which is precisely what a user id would be.

**Register drift (confirmed; same pattern as every other item so far):** the register cites `select_public_key_by_user_id/:41`. Line 41 is the `BUG(pinned)` doc comment inside the `#[cfg(test)]` module — the marker that *records* the misnomer. The defect proper is the module directory name plus the `pub mod` declaration at `public_keys/mod.rs:4`. The drift here is milder than elsewhere (the marker text and the defect describe the same thing), but the cited line is still the test-side annotation, not the offending identifier.

**One correction to the register's framing:** the register implies the *name to fix* spans trait/impl/tests/callers. In fact only the postgres-internal module path is dishonest. The repository trait (`SelectPublicKeyByIdQuery`), the SQL file, the tracing span, and the entire upstream stack — kernel `FindPublicKeyById`, services `FindPublicKeyByIdUseCase` (`oxidauth-services/src/public_keys/find_public_key_by_id.rs:3`, which imports the *correctly named* trait), the API handler (`oxidauth-api/src/server/api/v1/public_keys/find_public_key_by_id.rs`, wired at `oxidauth-api/src/provider/services.rs:563-569`), the `oxidauth-http` DTO, and the `oxidauth-rs` client — are all consistently `*_by_id`. The rename is smaller and safer than the register suggests.

## Analysis

**Mechanism / origin.** The impl file is a template copy: every sibling query module in `oxidauth-postgres` follows the same shape (`impl Service<&Query> for Pg*Repository`, `include_str!` of a same-named `.sql`). Whoever added this one named the *directory* after a by-user pattern (the `by_user_id` suffix is common elsewhere in the crate — five honest users of it, listed above) while the *SQL* was written honestly as a PK lookup. Nothing catches the mismatch because Rust resolves the query by the *request type* `&'a FindPublicKeyById`, not by module path: `oxidauth_repository::public_keys::select_public_key_by_id::SelectPublicKeyByIdQuery` is satisfied via the blanket impl (`oxidauth-repository/src/public_keys/select_public_key_by_id.rs:10-13`) the moment the `Service` impl exists anywhere in the postgres crate. The module name is load-bearing for *nobody's* code — which is exactly why it survived — and costs something only to *readers*.

**Who consumes the lying path today:** nobody. `pub mod select_public_key_by_user_id;` (`public_keys/mod.rs:4`) makes `oxidauth_postgres::public_keys::select_public_key_by_user_id` a public path, but workspace-wide `rg` finds zero references to it outside the crate's own module tree — services depend exclusively on the repository trait. The rename therefore has zero external-API impact; even the module's `pub` visibility could drop, though keeping `pub mod` matches the sibling convention (`delete_public_key`, `insert_public_key`, `select_all_public_keys` are all `pub mod`).

**Is a `user_id` link actually planned (the "document instead" question)?** No. Verified three ways: (1) the DDL has never had the column and no follow-up migration touches `public_keys`; (2) no ticket plans one — OXA-000017's work on this same table is uniqueness on *key material* (a `UNIQUE`/sha256 expression index over `public_key` bytes plus dedup), which is orthogonal to user ownership; (3) the domain model is user-free by design: keys are server-wide JWT signing keys — bootstrap creates the first one (`oxidauth-services/src/bootstrap/`), jwks serves them all (OXA-000012), and the only writer mints a fresh keypair per call. There is no per-user key concept anywhere in kernel, services, or the wire types. So "rename," not "keep the name and document intent."

**Edge cases / non-issues.**
- The `#[allow(deprecated)]` at `mod.rs:30` is the uniform crate-wide test-module convention — it silences the deprecated `oxidauth_kernel::service::Service` trait (`oxidauth-kernel/src/service.rs:4-9`, "generic request dispatch is retired … removed for 2.0"), which `mod.rs:7` implements and `use super::*` re-imports; it appears in all 46 test modules of the crate, is unrelated to this item, and is untouched by a rename.
- Behavior is *correct* as written: sanitized projection (`PgPublicSanitizedKey` never selects `private_key`; the test asserts non-leak via `Debug` at `mod.rs:56-62`), read-pool routing, `RowNotFound` on miss. Nothing here needs a behavior change; this ticket is naming hygiene only.
- P3 is honest severity: zero runtime impact, zero reachable-bug surface. The damage is searchability and reader trust: grepping `by_user_id` (as DATA-10's own investigation and any future user-scoping audit will do) returns a public_keys hit that is a decoy, and a future contributor could "fix" the SQL to match the name — adding `WHERE user_id = $1` against a column that doesn't exist — which sqlx's macro-free `query_as::<_>(include_str!(...))` would only catch at runtime, not compile time.

## Impact

- **Who is affected:** readers and auditors of `oxidauth-postgres` — any `by_user_id` grep, such as the one DATA-10's investigation implies, returns a public_keys hit that is a decoy — plus the register/ledger trail itself: `missing-tests.md:140,:473` and OXA-000017's cross-ref list (`OXA-000017-...:16`) carry the lying name into follow-up docs.
- **Production impact:** none. Wire format, HTTP routes, kernel/service types, trait names, SQL, and error behavior are all unchanged by the fix. No deploy coordination, no data migration.
- **Residual risk if not fixed:** the name keeps attracting the wrong "fix" and keeps polluting user-scoping audits of this table; cost of doing it properly is minutes.

## Proposed resolution

Rename-only. Three edits, no API surface touched:

1. `git mv oxidauth-postgres/src/public_keys/select_public_key_by_user_id oxidauth-postgres/src/public_keys/select_public_key_by_id` — the directory's `mod.rs` needs no content change (it imports the already-correct repository module and `include_str!`s the already-correct SQL filename).
2. `oxidauth-postgres/src/public_keys/mod.rs:4`: `pub mod select_public_key_by_user_id;` → `pub mod select_public_key_by_id;`.
3. Delete the now-stale `BUG(pinned)` marker at `select_public_key_by_id/mod.rs:41-43`. **No assertion flips**: the marker *documents the misnomer*, it does not pin wrong behavior — both tests (`it_should_find_the_sanitized_key_by_primary_id`, `it_should_error_when_the_key_is_missing`) call `.call(&FindPublicKeyById { public_key_id })` via `super::*` and assert PK semantics that the rename preserves exactly. This is the rare register item where the pinned test survives untouched and only its comment dies. Optionally rename the first test to drop "primary" emphasis; not required.

Ledger/doc follow-ups in the same change: tick `BUGS_AND_NOTES.md:43` as fixed, update the `missing-tests.md:140,:473` module references to the new name, and if OXA-000017's cross-ref line (`:16`) is still live when this lands, update its pointer to this module.

**Explicit non-goals:** no `user_id` column, no trait/query/DTO/wire rename (already correct), no visibility change on the module, no behavior change. Do not fold in DATA-13 (`select_all_public_keys` `ORDER BY`) or OXA-000017's uniqueness migration just because they share the directory/table — this ticket is mechanical and should stay that way.

## Verification

- `cargo check -p oxidauth-postgres` — the rename is crate-internal; a clean check proves the `pub mod` line and `include_str!` are the only in-crate couplings.
- `cargo check --workspace` (or `cargo test --no-run`) — proves zero external consumers of the old public path `oxidauth_postgres::public_keys::select_public_key_by_user_id` across services/api/kernel/http/rs-client crates.
- `DATABASE_URL=postgres://… cargo test -p oxidauth-postgres select_public_key_by_id` (sqlx tests need a live Postgres; `READ_DATABASE_URL` optional, falls back to `DATABASE_URL` per `oxidauth-postgres/src/lib.rs:9-10`; `MIGRATOR` at `lib.rs:45`) — both tests in the renamed module must pass unchanged, the same assertions as before the rename.
- `rg select_public_key_by_user_id src/` returns nothing; remaining hits confined to historical ledgers (`BUGS_AND_NOTES.md`, `missing-tests.md`) or updated with them.

## Decision (2026-09-29) — ACCEPTED as proposed, scheduled; no implementation started
- Reviewed jointly (workspace-wide rename blast-radius sweep + owner ruling 2026-09-29). Confirmed: the string exists only in the directory name + `pub mod` line at `public_keys/mod.rs:4`; zero external consumers (trait, kernel request, services, API handler, http DTO, client are all correctly `*_by_id`); `public_keys` never had a `user_id` column (single migration, ever). Rename-only, three edits.
- Coordinate re-verification at implementation: the `BUG(pinned)` doc comment sits at `select_public_key_by_user_id/mod.rs:41-43` — the ticket's original coordinate was correct; an earlier review-time "correction" to `:29-33` was wrong and is retracted. Implementer deleted it by content.
- **No assertion flips** — the rare register item where the pinned tests survive untouched and only the comment dies: both tests call `.call(&FindPublicKeyById { public_key_id })` and assert PK semantics the rename preserves exactly.
- Interlock with OXA-000052 (SCHEDULED): the span `select_public_key_by_id_query` (`mod.rs:11`) is true-to-SQL and MUST NOT be touched here; 000052 deliberately left it pending this rename. Directory + span become consistent as a pair.
- Sequencing: land early; OXA-000069's Service→Query rewrite rewrites this module's impl header, and a content-free `git mv` before it avoids path churn (either order compiles; before is cheaper).
- Non-goals kept as written: no `user_id` column, no trait/query/DTO/wire rename, no visibility change, no DATA-13/OXA-000017 folding.
- Ledger at implementation: tick `BUGS_AND_NOTES.md:43`, update `missing-tests.md:140/:473`, and OXA-000017's `:16` cross-ref pointer if still live.
- Acceptance: `cargo check --workspace` green (proves zero external consumers of the old path); `rg select_public_key_by_user_id src/` empty; both module tests pass unchanged against a live DB; `cargo test -p oxidauth-postgres select_public_key_by_id` is the post-rename name.
