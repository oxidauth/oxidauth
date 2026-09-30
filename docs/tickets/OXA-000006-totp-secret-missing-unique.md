# OXA-000006 — totp_secrets.user_id has no UNIQUE; duplicate enrollments make verification read an arbitrary secret

**Original ID:** SEC-6 · **Severity:** P2 · **Type:** bug · **Status:** open
**Review tier:** Tier 4 (moderate — multi-file recipes, pinned-test flips) · reviewed 2026-09-30 · DEFERRED (owner decision; claims verified intact at HEAD — dedup+UNIQUE migration + ON CONFLICT writers sound; arbitrary-secret verify-luck risk stands — revisit before any prod push; OXA-000005 Step-2 would sidestep the table long-term)


## Locations

Register line verified against current code; line numbers are accurate today.

- `src/oxidauth/oxidauth-postgres/migrations/20240416170202_create_totp_secrets.sql` — table DDL: `user_id UUID NOT NULL`, only a **non-unique** `CREATE INDEX totp_secrets_user_id` (line 10). No UNIQUE constraint anywhere; no later migration adds one (checked all 18 files in `oxidauth-postgres/migrations/`).
- `src/oxidauth/oxidauth-postgres/src/totp_secrets/insert_totp_secret/mod.rs:82` — `BUG(pinned)` marker inside `it_should_allow_duplicate_secrets_per_user` (:81–105).
- `src/oxidauth/oxidauth-postgres/src/totp_secrets/insert_totp_secret/insert_totp_secret.sql` — plain `INSERT … RETURNING *`, no `ON CONFLICT`.
- `src/oxidauth/oxidauth-postgres/src/totp_secrets/select_totp_secret_by_user_id/mod.rs:85` — `BUG(pinned)` marker inside `it_should_return_one_secret_when_the_user_has_several` (:84–101); `fetch_one` at mod.rs:34.
- `src/oxidauth/oxidauth-postgres/src/totp_secrets/select_totp_secret_by_user_id/select_totp_secret_by_user_id.sql` — `SELECT * FROM totp_secrets WHERE user_id = $1`, no `ORDER BY`/`LIMIT`.
- Duplicate-producing writers:
  - `src/oxidauth/oxidauth-services/src/user_authorities/create_user_authority.rs:67-75` — every registration against a TOTP-enabled authority mints a fresh secret unconditionally; secret insert runs **before** the `insert_user_authority` call (:77–79) with no shared transaction.
  - `src/oxidauth/oxidauth-services/src/authorities/update_authority.rs:71-77` — `Disabled → Enabled` flip bulk-provisions secrets.
  - `src/oxidauth/oxidauth-postgres/src/totp_secrets/select_where_no_totp_secret_by_authority_id/select_where_no_totp_secret_by_authority_id.sql` — anti-join guard, **no `DISTINCT`**.
  - `src/oxidauth/oxidauth-postgres/src/totp_secrets/insert_totp_secrets/mod.rs:24-37` — loops `insert_totp_secret_query` per pair inside one tx.
- Consumer sites (all verify against the arbitrarily-selected secret): `oxidauth-services/src/totp/validate.rs:102-122` (issues JWT on success), `auth/authenticate.rs:178-182`, `strategies/username_password/update_password.rs:113-117`, `strategies/username_password/forgot_password.rs:55-59`.
- Unrelated markers checked and excluded: `oxidauth-rs/src/client/users/contract.rs:319` (TotpSettings serialization) and `oxidauth-services/src/bootstrap/mod.rs:1641` (permissions bootstrap) — not about duplicate enrollment.

## Problem

`totp_secrets` allows any number of rows per user. Once duplicates exist, `select_totp_secret_by_user_id_query` uses sqlx `fetch_one`, which returns the **first row of a multi-row result set without erroring** — with no `ORDER BY`, the winner is plan-dependent (unspecified). Every TOTP verification path (`totp/validate.rs`, the TOTP step in `authenticate.rs`, `update_password.rs`, `forgot_password.rs`) then checks the user's code against whichever secret the planner happened to surface, silently and nondeterministically. The register's `:82` / `:85` point at the two `BUG(pinned)` tests that codify this behavior; both markers are present at exactly those lines today.

## Analysis

**Root cause:** schema-level missing invariant. `create_totp_secrets_by_authority_id.rs:63-66` mints `random_string()` per selected user id and inserts; the single-enroll use case `create_totp_secret.rs:43-52` does the same per call. Neither has any existence check or conflict handling, and nothing in the database rejects the second row.

**Concrete duplicate sources (verified in code):**

1. **Re-registration / multiple TOTP authorities.** `create_user_authority.rs:67-75` inserts a secret on *every* registration while `TotpSettings::Enabled`, unconditionally. A user granted under two TOTP-enabled authorities, or removed from an authority (the `user_authorities` FK cascade deletes the grant; `totp_secrets` rows are user-keyed, so an admin-side grant deletion — no user delete — leaves them) and re-registered, accumulates rows.
2. **Secret inserted before the grant, no shared transaction.** In `create_user_authority.rs` the secret insert (:72–74) precedes `insert_user_authority` (:77–79). When the grant insert fails — e.g. `UNIQUE (user_identifier, authority_id)` from `20240619063635` — the already-persisted secret is not rolled back. Repeated failed registration attempts each leave a fresh orphan secret behind.
3. **Bulk path self-duplicates inside one transaction.** The anti-join SQL selects `user_authorities.user_id` with no `DISTINCT`. A user with two identifiers under the same authority (legal, since uniqueness is `(user_identifier, authority_id)`, not `user_id`) yields that `user_id` twice; `insert_totp_secrets` then inserts two rows for the same user in a single tx — both with the same `NOW()` (`now()` is transaction timestamp, so equal within the tx).
4. **Check-then-insert race.** `update_authority`'s Disabled→Enabled provision is a select-then-insert across two service calls; two concurrent `update_authority` requests (or a concurrent `create_user_authority`) both pass the anti-join and both insert.

**Why "arbitrary" matters:** sqlx `fetch_one` executes the query and takes `rows.next()` — a multi-row result is *not* an error (`RowNotFound` only fires on zero rows, as the pinned no-secret test shows). Absent `ORDER BY`, PostgreSQL gives no row-order guarantee; in practice a `totp_secrets_user_id` index scan returns rows in insertion/ctid order (oldest first), but that is not contract.

**Related but out of scope:** the select runs on `read_pool()` while inserts use `write_pool()` (`select_totp_secret_by_user_id/mod.rs:14`, `insert_totp_secret/mod.rs:17`); replica lag can serve stale rows independently of this bug. Note also that secrets are never client-visible: `CreateTotpSecretResponse` is `{ success: true }` only, and `forgot_password` hands out *codes* generated from the stored secret, not the secret itself — this materially eases migration (see below).

## Impact

- **MFA is verified against an unspecified credential.** With N rows, only one secret is ever checked; the other N−1 are live-but-invisible credentials. Any secret ever created for the user (including ones left by failed registrations, source 2) can become the effective one, and nothing expires or revokes superseded rows. Conversely, if the planner picks an older row, the user's newest issued codes (generated from the newest row by `forgot_password`) fail with "invalid totp code".
- **Nondeterministic pairing in recovery flows.** `forgot_password.rs:55-59` mints a code from the fetched row; `update_password.rs:113-117` later re-fetches to validate. With duplicates (or replica lag), the two fetches may pick different rows and the recovery code fails at random — same class of failure, users can be locked out of password recovery.
- **Who is affected:** every deployment with a TOTP-enabled authority (`TotpSettings::Enabled`) where any user holds ≥2 rows — multi-authority users, re-registered users, multi-identifier users, or anyone provisioned during a concurrent authority update. `totp/validate.rs` success mints the session JWT (:124+), so the wrong-secret acceptance is an authentication-boundary issue, not just data hygiene.

## Proposed resolution

1. **New migration** `oxidauth-postgres/migrations/<timestamp>_add_unique_totp_secret_per_user.sql` — resolve duplicates first, deterministically keep-latest, then constrain:

   ```sql
   -- One row survives per user; tie-break on id so the choice is deterministic
   -- and the migration is idempotent under replay.
   DELETE FROM totp_secrets ts
   WHERE EXISTS (
       SELECT 1 FROM totp_secrets newer
       WHERE newer.user_id = ts.user_id
         AND (newer.created_at, newer.id) > (ts.created_at, ts.id)
   );

   ALTER TABLE totp_secrets
       ADD CONSTRAINT totp_secrets_user_id_key UNIQUE (user_id);

   -- Now redundant: UNIQUE builds its own btree index.
   DROP INDEX totp_secrets_user_id;
   ```

   Keep-latest rationale: codes are always re-generated from the stored row at request time and the secret never leaves the server, so deleting older rows makes in-flight older *codes* invalid (they expire by TTL anyway) while newer ones keep working — near-zero user-visible impact. A down-migration can drop the constraint but cannot restore deleted rows; acceptable, note it in the migration header. sqlx runs each migration file in a transaction, so the build takes an `ACCESS EXCLUSIVE` lock briefly; `totp_secrets` is small (one row per user) — if a deployment ever needs lock-free builds, split into a non-transactional `CREATE UNIQUE INDEX CONCURRENTLY` + `SET NOT NULL`-style dance, out of scope here.

2. **Single-enroll insert** — rotation semantics on re-enroll. `insert_totp_secret.sql`:

   ```sql
   INSERT INTO totp_secrets
   (id, user_id, totp_secret, created_at)
   VALUES (uuid_generate_v4(), $1, $2, NOW())
   ON CONFLICT (user_id) DO UPDATE
   SET totp_secret = EXCLUDED.totp_secret,
       created_at  = EXCLUDED.created_at
   RETURNING *
   ```

   `RETURNING *` works unchanged under `DO UPDATE`, so `fetch_one` and `PgTotpSecret` mapping are untouched. Rejected alternative `DO NOTHING`: silently keeps the stale secret while the caller reports a fresh enrollment — misleading. Plain `INSERT` (error on conflict) would surface `23505` as raw sqlx text through `BoxedError` at a user-facing registration path; if product decides re-enroll must be *rejected* with a domain error instead of rotating, map the conflict in `insert_totp_secret/mod.rs` to a typed error rather than leaving it raw.

3. **Bulk insert** — `insert_totp_secrets/mod.rs` must tolerate conflicts, otherwise the select-then-insert race from step 4 above turns from silent duplicates into a whole-tx `23505` abort once the constraint lands. Simplest correct change: route the bulk path through a variant SQL with `ON CONFLICT (user_id) DO NOTHING` (the anti-join already means "never touch users who have one"); alternatively reuse the single statement since the batch selects only users lacking a secret. Add `DISTINCT` to `select_where_no_totp_secret_by_authority_id.sql` to kill the in-transaction self-duplicate from multi-identifier users.

4. **Order fix** in `create_user_authority.rs`: move the `create_totp_secret` call after a successful `insert_user_authority` (or wrap both repos in one transaction — bigger refactor, flag separately). Without the reorder, the rotate-upsert would let a *failed* registration wipe the user's current valid secret via source 2's path.

5. **Select path:** leave `select_totp_secret_by_user_id.sql` as-is once the constraint exists — the invariant is now database-enforced; `ORDER BY … LIMIT 1` would only paper over the missing constraint.

6. **Pinned-test flips** (the two markers above; the `contract.rs:319` and `bootstrap/mod.rs:1641` markers are unrelated — do not touch):
   - `insert_totp_secret/mod.rs:81-105` `it_should_allow_duplicate_secrets_per_user` → rewrite as `it_should_rotate_the_secret_on_reenrollment`: second `repo.call(&params)` returns `Ok`, `COUNT(*) = 1`, surviving `totp_secret` equals the second insert's key. Drop the `BUG(pinned)` comment.
   - `select_totp_secret_by_user_id/mod.rs:84-101` `it_should_return_one_secret_when_the_user_has_several` → the `seed_totp_secret` ×2 setup becomes impossible; replace with a constraint test: raw second `sqlx::query("INSERT INTO totp_secrets …")` must fail, asserted via the existing `test_fixtures::assert_sql_state` helper with SQLSTATE `23505`. (`oxidauth_services::user_authorities::create_user_authority` and `authorities::update_authority` unit tests use mocks and are unaffected.)

## Verification

- Migration replay on a DB seeded with duplicates (two rows for one user, one pair with identical `created_at` to exercise the `id` tie-break): `sqlx migrate run` then
  `SELECT user_id, COUNT(*) FROM totp_secrets GROUP BY user_id HAVING COUNT(*) > 1` → **0 rows**, and `\d totp_secrets` shows `totp_secrets_user_id_key` UNIQUE with the old non-unique index gone.
- `cargo test -p oxidauth-postgres totp_secrets` — flipped insert/select tests above plus existing `it_should_insert_a_totp_secret`, `it_should_reject_an_unknown_user`, the `insert_totp_secrets` transactional-rollback tests, and `select_where_no_totp_secret_by_authority_id` (needs a new case: same user with two identifiers in the authority appears once after `DISTINCT`). Requires the project's postgres test URL (`DATABASE_URL` / `.env`, per `#[sqlx::test(migrator = "crate::MIGRATOR")]`).
- `cargo test -p oxidauth-services totp` and `cargo test -p oxidauth-services user_authority authority` — `create_user_authority` ordering change and bulk-provision path keep the mock-based suites green; add one service-level regression: registering an existing TOTP user must not error and must rotate, not append.
- End-to-end smoke against a live server: enable TOTP on an authority via `update_authority`, re-register a user under a second TOTP authority, then confirm exactly one `totp_secrets` row and that `totp/validate` succeeds for the surviving secret.
