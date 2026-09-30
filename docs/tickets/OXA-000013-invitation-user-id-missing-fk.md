# OXA-000013 — invitations.user_id is indexed but never a FK; dangling invitations accepted

**Original ID:** SEC-13 · **Severity:** P3 · **Type:** bug · **Status:** implemented 2026-09-30 (coder+reviewer approved; migration 20260930180000 live — fresh-DB + pre-orphan scratch proofs green, reviewer re-ran independently; sqlx-silent-SELECT logging caveat corrected in migration+changelog; live-API smoke §4 left to Main post-merge)
**Review tier:** Tier 4 (moderate — multi-file recipes, pinned-test flips) · reviewed 2026-09-30 · SCHEDULED (owner approved; scope: migration — DELETE dangling invitations + `invitations_users_fk ... ON DELETE RESTRICT` (deliberate: CASCADE would silently eat pending invites; operators revoke first), flip pin to 23503-reject + RESTRICT-semantics test, release note for delete-user 400s; zero Rust changes)


## Locations

Register path `oxidauth-postgres/src/invitations/insert_invitation/:101` points at the `BUG(pinned)` marker comment inside the test module, not at the defect. Verified current locations:

- **The defect:** `src/oxidauth/oxidauth-postgres/migrations/20240127042304_create_invitations.sql` — :3 `user_id UUID NOT NULL` (bare column, no `REFERENCES users(id)`), :9 `CREATE INDEX invitations_user_id ON invitations(user_id);`. Indexed, not constrained. Nothing in any later migration adds an FK (the migrations dir has 18 files; only `20240523190325`, `20240619063635`, `20250610213132` follow the invitations table and touch authorities/settings, verified by read of the directory and file names).
- **The pin (register's `:101`):** `src/oxidauth/oxidauth-postgres/src/invitations/insert_invitation/mod.rs` — test `it_should_accept_an_unknown_user_because_the_column_has_no_fk` at :99–114; `BUG(pinned)` comment at :101–103; `.expect("no FK exists on invitations.user_id, so this insert passes")` at :111.
- **Insert path:** `oxidauth-postgres/src/invitations/insert_invitation/insert_invitation.sql` (`INSERT … (id, user_id, expires_at, …) VALUES (COALESCE($1, uuid_generate_v4()), $2, $3, NOW(), NOW()) RETURNING *`); executor `mod.rs:17–22` (`fetch_one` on the write pool); params `oxidauth-repository/src/invitations/insert_invitation.rs:17–21` — `user_id: Uuid` is non-nullable at the type level, so the column is never NULL, only ever *unverified*.
- **Sole API caller:** `oxidauth-services/src/invitations/create_invitation.rs:48–72` — `CreateInvitationUseCase` creates the user **first** (:52–55) and inserts the invitation with `user_id: user.id` (:59). `CreateInvitationParams` carries `user: CreateUser` (`oxidauth-kernel/src/invitations/create_invitation.rs:31–35`), no raw `user_id` field — so no HTTP path can supply an arbitrary user id. Endpoint `POST /v1/invitations` is authenticated + permissioned (`oxidauth-api/src/server/api/v1/invitations/create_invitation.rs:15` `oxidauth:invitations:create`, `mod.rs:14`).
- **Consumer that makes dangling rows dangerous:** `oxidauth-services/src/invitations/accept_invitation.rs:52–86` — claims the invitation by *deleting* it (:53–60), then `create_user_authority` with `user_id` from the invitation row (:62–76), then `update_user` with the same `user_id` (:78–83). No existence check on the user anywhere in the flow.
- **Dangling-row producer:** `DELETE /users/{user_id}` (`oxidauth-api/src/server/api/v1/users/mod.rs:29`) → `DeleteUserByIdUseCase` (pure delegation, `oxidauth-services/src/users/delete_user_by_id.rs`) → hard delete `oxidauth-postgres/src/users/delete_user_by_id_query/delete_user_by_id_query.sql` (`DELETE FROM users WHERE id = $1 RETURNING *`). Errors map to HTTP 400 at `oxidauth-api/src/server/api/v1/users/delete_user_by_id.rs:59`.
- **Convention this table violates:** every other user referencing table declares the FK — `user_authorities` (`user_authorities_users_fk`), `user_role_grants` / `role_*_grants`, `refresh_tokens` (`refresh_tokens_users_fk`), `totp_secrets` (`totp_secrets_users_fk`), all `REFERENCES users(id) ON DELETE CASCADE` (verified in the migration files). `invitations` is the only `user_id` column without one.
- **Test fixtures:** `seed_user` at `oxidauth-postgres/src/test_fixtures.rs:106`, FK-violation helper `assert_sql_state` at :91–103; precedent test `it_should_reject_unknown_user_or_role_ids` in `oxidauth-postgres/src/user_role_grants/insert_user_role_grant/mod.rs` (asserts SQLSTATE `23503` at :110, :119).

## Problem

`invitations.user_id` is `NOT NULL` and indexed but has no foreign key to `users(id)`. The repository layer will persist an invitation for any UUID, existing or not — the pinned test proves it by inserting `user_id: Uuid::new_v4()` and expecting success. Referential integrity for the table rests entirely on one caller (`CreateInvitationUseCase`) happening to create the user first, with no DB-level enforcement of that contract.

Two ways the table goes dangling, verified in code:

1. **User deletion after invitation creation** — the real-world path. `DELETE /users/{id}` hard-deletes the user; Postgres cascades through authorities/grants/refresh tokens/TOTP, but nothing touches `invitations`, leaving an invitation pointing at a nonexistent user.
2. **Any non-service writer** — the repository API is `pub` and accepts an arbitrary `Uuid`; direct SQL/maintenance scripts, or any future "invite an existing user" feature that reuses `InsertInvitationQuery`, silently write garbage.

Once an invitation is dangling, the accept flow (`POST /v1/invitations/{id}`, unauthenticated bearer claim, `invitations/mod.rs:16`) fails *partway through*: the DELETE claims the row successfully, then `create_user_authority` hits `user_authorities_users_fk` (`23503` — that table *does* have the FK), the whole call surfaces as a 400 to the invitee, and the invitation is already consumed. Admins see a live ghost invitation in `GET /v1/invitations/{id}` until somebody tries to accept it.

## Analysis

**The register claim is accurate, with one nuance.** "Dangling invitations … accepted" is literally true at the repository layer (that's what the pin asserts). Via the HTTP API the column is *de facto* valid **at insert time** — the only caller derives `user_id` from a user it just created. The defect is therefore about *durability* of the invariant (user deletion later) and *enforceability* (the DB doesn't uphold what the service promises), not about an exploitable API hole today.

**The accept-flow failure ordering (verified line by line):** claim deletes the invitation → `create_user_authority` inserts into `user_authorities`, whose FK to `users` rejects the dangling `user_id` → `Err` propagates out of the use case before `update_user` ever runs. Consequences: (a) the invitee's token is destroyed with no recoverable state — re-accepting returns `RowNotFound` → 400 forever; (b) cleanup requires manual SQL, because the *invitation* is gone and the *user* is gone; only the audit trail survives. (The general non-transactionality of accept is already noted in OXA-000002 §Analysis; this ticket's FK removes its most likely trigger — see below.)

**RESTRICT vs CASCADE — the product decision.** Sibling tables all use `ON DELETE CASCADE`, and the reflex is to copy that. For `invitations` it's the wrong default, for reasons specific to the invite→user lifetime:

- The invitation row *is* the claim token/bearer capability for the placeholder user (`user_id` points at a user row minted by `create_invitation` that only becomes a real account when accept overwrites it). Under CASCADE, holding `oxidauth:users:delete` would implicitly revoke pending offers — an entitlement cross-leak against `oxidauth:invitations:delete` (`delete_invitation.rs:18`), and a silent destruction of an outstanding-access record.
- Under CASCADE, user deletion *racing* an accept still produces the partial failure above (invitation deleted, then user deleted, then authority insert fails). Under RESTRICT the interleaving is almost impossible: while the invitation exists the user cannot be deleted; only a delete issued *inside* the accept request's own sub-request window could reproduce it, narrowing a guaranteed bug to a vanishing race. (Full serialization of accept is OXA-000002-adjacent transactionality work; not this ticket.)
- The friction is coherent, not annoying: the only users RESTRICT blocks from deletion are those with *unaccepted* invitations — i.e., exactly the placeholder rows created for that offer. The prescribed order (revoke invitation via the permissioned `DELETE /v1/invitations/{id}`, then delete the user) is already the correct two-step cleanup, and accepted users keep no `invitations` row at all, so no real account is ever blocked.

**Compat consequence of RESTRICT:** on existing databases, `DELETE /users/{id}` for a user with a pending invitation starts failing (sqlx `23503` surfaces as HTTP 400 `bad_request`, `delete_user_by_id.rs:59`, with the constraint name in the error payload). Behavior change for ops; acceptable and self-explanatory, but it must be stated in release notes. If product rejects that friction, CASCADE is a one-word drop-in with identical cleanup — record that as the explicit alternative, not a silent fallback.

**Migration preconditions:** `ALTER TABLE … ADD CONSTRAINT … FOREIGN KEY` validates existing rows, so the migration fails (`23503` during validation) unless dangling rows are removed first. Dedupe isn't a concern (`id` is PK and `user_id` dangles are per-row, not conflicting), but the cleanup DELETE should be logged (count `SELECT` first — Postgres output lands in the migration log). No upcast is possible: an invitation for a nonexistent user cannot be repaired, only discarded. The existing `invitations_user_id` index already covers what Postgres needs to check child rows on user delete — keep it, don't add a duplicate. The SQLSTATE on the Rust side is `23503 foreign_key_violation` surfaced as `sqlx::Error::Database` inside `BoxedError` — identical to every existing FK in the codebase, so callers/mocks need no changes. No query changes are required in Rust at all: the insert SQL stays as-is; Postgres just starts rejecting bad values, same as `user_role_grants` inserts do today.

**Related but distinct, do not conflate:** OXA-000002/SEC-2 (expired invitations accepted; `BUG(pinned)` at `accept_invitation.rs:385`) — same table, different bug, its pin must stay. CLI-6 (`oxidauth-rs` `METHOD` constant markers at `client/invitations/create_invitation.rs:73`, `find_invitation.rs:75`) — cosmetic, unrelated.

**Pinned markers touching this item:** exactly one — `oxidauth-postgres/src/invitations/insert_invitation/mod.rs:101` (verified via `grep -rn 'BUG(pinned)' src/oxidauth --include='*.rs'`). No hurl coverage exists for invitations (confirmed while writing OXA-000002), so the unit pin is the only test encoding this bug.

## Impact

- **Affected:** self-hosted deployments that ever delete users (`DELETE /users/{id}` is a shipped, permissioned endpoint). `invitations` accumulates permanently dangling rows — nothing in the codebase ever deletes by expiry or by missing user (verified: no scheduler, cleanup only consumes what accept/delete touch), so the table grows with ghost offers that `find_invitation` still serves.
- **UX:** invitees with a since-deleted target user get an unrecoverable 400 *after* their token is consumed; admins cannot re-issue without recreating user+invitation, and have no SQL-free way to understand the state.
- **Data-integrity/latent:** the repository API advertises `user_id: Uuid` with no validity contract; every future consumer must re-implement "create the user first" by convention. The FK makes the invariant enforceable rather than customary.
- **Not a security escalation:** dangling invitations cannot take over anything — `user_authorities`' own FK blocks the credential grant. P3 is correctly calibrated: integrity/UX/data-hygiene, no privilege gain.

## Proposed resolution

**1. New migration** `src/oxidauth/oxidauth-postgres/migrations/<timestamp>_add_invitations_users_fk.sql`:

```sql
-- Clean up rows the missing FK let in. An invitation for a nonexistent user
-- cannot be repaired, only discarded. Log the count before deleting.
DELETE FROM invitations i
WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = i.user_id);

ALTER TABLE invitations
    ADD CONSTRAINT invitations_users_fk
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE RESTRICT;
```

- Constraint name follows the sibling convention (`<table>_users_fk`).
- `RESTRICT` (immediate, non-deferrable) rather than the siblings' `CASCADE` — rationale in Analysis; keep the decision visible in the migration comment since it deviates from the house default.
- Keep `invitations_user_id` (:9) untouched; it serves the FK's delete-check.
- Very large deployments may prefer `ADD CONSTRAINT … NOT VALID` followed by `VALIDATE CONSTRAINT` in a separate step to shorten the lock window; the repo's migrations are simple serial scripts and `invitations` is small by nature (one row per pending offer), so the simple form above is recommended.
- No Rust changes: `insert_invitation.sql`, `PgInvitationRepository`, services, API wiring, and `oxidauth-http` DTOs all stay byte-identical; violations surface through the existing `BoxedError` path exactly like `user_role_grants` FK violations.

**2. Release note / ops guidance:** `DELETE /v1/users/{id}` for a user with a pending invitation now returns 400 (`invitations_users_fk` in the error). Documented runbook: revoke the invitation first (`DELETE /v1/invitations/{id}`, `oxidauth:invitations:delete`), then delete the user. If product rejects the friction, CASCADE is the drop-in alternative with the same cleanup step — a flagged product decision, not a code fork.

**3. Pinned-test handling (flip when fixing):**
- `oxidauth-postgres/src/invitations/insert_invitation/mod.rs:99–114` — rewrite `it_should_accept_an_unknown_user_because_the_column_has_no_fk` as `it_should_reject_an_unknown_user_because_of_the_users_fk`: remove the `BUG(pinned)` comment (:101–103), replace `.expect(...)` with `expect_err` + `assert_sql_state(err, "23503")` (`test_fixtures.rs:91`), and assert `COUNT(*) FROM invitations == 0`. Mirror `insert_user_role_grant/mod.rs`'s `it_should_reject_unknown_user_or_role_ids` (asserts at :110/:119).
- **Add** a new test in the same module pinning the RESTRICT choice: `seed_user` + insert invitation, then raw `DELETE FROM users WHERE id = $1` → `assert_sql_state(err, "23503")` and the invitation row still exists. This prevents a future silent switch to CASCADE.
- Everything else stays green **as-is**: the two happy-path insert tests (:46–97) and both `select_invitation_by_id` / `delete_invitation_by_id` seeds all use `seed_user`, verified — the FK accepts them unchanged.
- **Do not touch** neighboring pins: `accept_invitation.rs:385` belongs to SEC-2/OXA-000002; `oxidauth-rs` client markers belong to CLI-6.

**4. Compat:** no API shape change, no DTO change, no data loss except pre-existing dangling rows (which were already unusable — accepting one only consumed it and failed). Existing invitations whose users still exist are untouched; their accept behavior is unchanged.

## Verification

1. Targeted integration tests (live docker postgres, project convention): `set -a && source .env && set +a && export MIGRATIONS_ENABLED=true && cargo test -p oxidauth-postgres -- invitations::` — flipped pin (`23503` rejection), new RESTRICT-on-user-delete test, and the four untouched tests all pass. Then `cargo test -p oxidauth-postgres -- users::` and `cargo test -p oxidauth-services -- invitations::` (service tests use mocks; both must be unaffected).
2. Schema check on a dev DB post-migration: `psql` `\d invitations` shows `invitations_users_fk` foreign key (users.id) ON DELETE RESTRICT *and* the retained `invitations_user_id` index; re-running `cargo test` from a fresh volume confirms the cleanup DELETE leaves `ADD CONSTRAINT` validation succeeding.
3. SQL-level repro: `INSERT INTO invitations (user_id, expires_at) VALUES (gen_random_uuid(), NOW() + interval '1 day')` → `foreign key "invitations_users_fk" violated` (`23503`); `DELETE FROM users` for a user holding a pending invitation → `23503`, invitation survives; deleting the invitation first, then the user → both succeed.
4. End-to-end smoke against the running API: admin JWT `POST /v1/invitations` → `DELETE /v1/users/{user_id}` → expect 400 naming `invitations_users_fk`; then `DELETE /v1/invitations/{id}` → 200, `DELETE /v1/users/{user_id}` → 200; with no user deletion in between, the normal accept flow still succeeds and consumes the row.
5. Marker hygiene: `grep -rn 'BUG(pinned)' src/oxidauth --include='*.rs'` → the `insert_invitation/mod.rs:101` marker is gone; the `accept_invitation.rs:385` and `oxidauth-rs` markers remain.
