# OXA-000002 — Expired invitations are accepted forever

**Original ID:** SEC-2 · **Severity:** P1 · **Type:** bug · **Status:** open
**Review tier:** Tier 4 (moderate — multi-file recipes, pinned-test flips) · reviewed 2026-09-30 · DEFERRED (owner decision; claims verified intact at HEAD — delete_valid_invitation_by_id fix plan sound, pairs with OXA-000013 — revisit before any prod push)


## Locations

Register path is stale: all crates live under `src/oxidauth/` (e.g. `oxidauth-services/...` → `src/oxidauth/oxidauth-services/...`), and `:385` points at the `BUG(pinned)` marker inside the unit-test module, not at the defective product code. Verified current locations:

- `src/oxidauth/oxidauth-services/src/invitations/accept_invitation.rs` — the use case. `accept_invitation` product code at :52–86 (no expiry comparison); pinned test `expired_invitation_is_still_accepted` at :383–400 with `BUG(pinned)` marker at :385.
- `src/oxidauth/oxidauth-postgres/src/invitations/delete_invitation_by_id/delete_invitation_by_id_query.sql` — `DELETE FROM invitations WHERE id = $1 RETURNING *`; no `expires_at` predicate.
- `src/oxidauth/oxidauth-postgres/src/invitations/delete_invitation_by_id/mod.rs` — `PgInvitationRepository: Service<&DeleteInvitationParams>` executing that SQL (:20–28).
- `src/oxidauth/oxidauth-repository/src/invitations/delete_invitation_by_id.rs` — blanket trait `DeleteInvitationByIdQuery` (auto-implemented for every `Service<&DeleteInvitationParams, Response = Invitation>`).
- Shared consumer (why the SQL can't just be filtered): `src/oxidauth/oxidauth-services/src/invitations/delete_invitation.rs:35–42` — `DeleteInvitationUseCase` is pure delegation to the *same* query, wired to the same `PgInvitationRepository` in `src/oxidauth/oxidauth-api/src/provider/services.rs:701–721`.
- Public entry point: `src/oxidauth/oxidauth-api/src/server/api/v1/invitations/mod.rs:16` (`POST /invitations/{invitation_id}`) → `accept_invitation.rs` handler (any `Err` → `Response::bad_request()`, :46–53). Create/find/delete handlers carry `ExtractJwt` + `parse_and_validate(PERMISSION, …)`; the accept handler has neither — it is unauthenticated.
- Expiry producer: `src/oxidauth/oxidauth-services/src/invitations/create_invitation.rs:60–63` (default `Utc::now() + 7 days`, TODO pivotal 186949366); DTO re-exports kernel `CreateInvitationParams` (`src/oxidauth/oxidauth-http/src/invitations/create_invitation.rs:8–10`), whose `expires_at: Option<DateTime<Utc>>` (`oxidauth-kernel/src/invitations/create_invitation.rs:33`) is caller-settable.
- Schema/entity: `src/oxidauth/oxidauth-postgres/migrations/20240127042304_create_invitations.sql` (`expires_at TIMESTAMPTZ NOT NULL`, no accepted/used-state column); kernel entity `Invitation { id, user_id, expires_at, … }` at `src/oxidauth/oxidauth-kernel/src/invitations/mod.rs:10–17`.

## Problem

An invitation that expired any amount of time ago can still be accepted, with full effect: the invited user account gets a new authority credential and its profile/username/status overwritten. The register's claim is accurate as verified today:

1. `AcceptInvitationUseCase::accept_invitation` builds `DeleteInvitationParams { id }` and calls the delete query; it destructures `let Invitation { user_id, .. }` (:57–60) — `expires_at` is loaded into memory (via `RETURNING *`) and discarded. Nothing anywhere on the path compares it to now.
2. The SQL is a bare `DELETE … WHERE id = $1 RETURNING *`. `sqlx::fetch_one` yields `RowNotFound` only when no row matches at all — expiry is invisible.
3. The use case then proceeds unconditionally: `create_user_authority` (attacker-chosen `client_key` + `params`, i.e. a password credential via `RegisterParams`) and `update_user` (username, email, names, status, profile).

The register's `:385` is the line of the `BUG(pinned)` marker inside the test module — the marker was recorded when the test suite pinned the buggy behavior (missing-tests.md Review Log S9: "`expired_invitation_is_still_accepted` (accept_invitation.rs:384) pins BUG(pinned) — a week-expired invitation completes all three steps").

## Analysis

**Delete-as-claim design.** The schema has no `accepted` column; accepting *is* deleting (missing-tests.md S9 item (4): "the id IS the bearer token"). Re-accept/replay therefore lands on `RowNotFound`, which is pinned by `missing_invitation_surfaces_the_repository_error_without_side_effects` (:402–419). Any fix must preserve that claim atomicity — a check-after-delete leaks an already-consumed row, and a select-then-delete reintroduces a (benign but sloppy) TOCTOU window.

**The shared-query trap.** `delete_invitation_by_id.sql` serves two masters: the public accept flow *and* the admin `DELETE /invitations/{id}` endpoint (`oxidauth:invitations:delete`). Adding `AND expires_at > NOW()` to that file would make expired invitations **undeletable by admins** (RowNotFound → 400) — expired rows would become permanent, un-revocable garbage. The fix needs a *separate* claim query.

**Unauthenticated bearer capability.** `POST /v1/invitations/{id}` has no `ExtractJwt` and no permission check (contrast create/find/delete in the same module directory). Security rests entirely on the v4-UUID being unguessable plus the expiry TTL. Expiry is the only time bound on a capability that grants full control of a pre-created account; "expires at" currently means "never".

**Who/how affected.** Any expired invitation link: forwarded/expired email, screenshots, logs, a revoked offer, an ex-employee with an old email archive. Combined with `create_invitation` accepting a caller-supplied `expires_at` with **no validation** (a past timestamp is stored as-is; there is no `expires_at > now` check in the service, SQL, or schema), an admin can even mint already-expired invitations today.

**Error surface.** The API maps every service error to HTTP 400 with the `OxidAuthError` envelope (`xlib/http` has no `Response::not_found()` builder — verified: only `success/unauthorized/internal_error/bad_request/fail`). Missing invitation today → 400 with `debug: "RowNotFound"`. Expired-after-fix can reuse the same observable without information loss (arguably a feature: the unauthenticated endpoint must not oracle "this invitation existed but expired" vs "never existed").

**Clock.** Repo precedent (`oxidauth-services/src/refresh_tokens/exchange_refresh_token.rs:101–124`) compares `expires_at.timestamp()` to Rust-side epoch in the service and deletes the token on expiry. For invitations the predicate can live in SQL (`NOW()`), which is consistent with `insert_invitation.sql` using `NOW()` for timestamps and keeps the claim atomic; DB-clock-vs-app-clock skew is milliseconds, immaterial against a days-scale TTL.

**Related but distinct, do not conflate:**
- SEC-13 (`BUGS_AND_NOTES.md:27`, marker `oxidauth-postgres/src/invitations/insert_invitation/mod.rs:101`): `invitations.user_id` has no FK. Same table, different bug — its pin must stay.
- CLI-6 (`oxidauth-rs` client `METHOD` typo markers): cosmetic, unrelated.
- Accept deletes the row *before* `create_user_authority`/`update_user` run; if those fail, the invitation is consumed with no account (pinned at :421–451). A real flaw adjacent to this one, but fixing transactionality is a separate change (note it, don't silently scope-creep).

**Pinned markers touching this item** (`grep -rn 'BUG(pinned)' src/oxidauth --include='*.rs'`): exactly one — `src/oxidauth/oxidauth-services/src/invitations/accept_invitation.rs:385`. No hurl coverage exists for invitations at all (`src/oxidauth/hurl/tests/` has none; verified by glob), so the unit pin is the only test asserting the bug.

## Impact

- **Security (P1):** expired invitation tokens grant account takeover of the invited user: attacker sets the authority credential (e.g. username/password), claims username/email, and can set `status` on a pre-created account — indefinitely, since nothing ever trims `expires_at` past. The documented product contract ("invitations expire", default 7 days) is silently false.
- **Data hygiene:** expired rows are never removed by any code path (accept deletes only what it consumes; nobody deletes by expiry), so `invitations` grows monotonically — worsened by the missing FK (SEC-13).
- **Trust/UX:** admins believe invitations expire; revoked-but-not-deleted offers remain live.

## Proposed resolution

**Primary: atomic expiry-filtered claim query (SQL predicate, new query — do NOT touch the shared SQL).**

1. **Kernel** (`oxidauth-kernel/src/invitations/`): add a distinct params type, e.g. `accept_invitation::ClaimInvitationParams { pub id: Uuid }`. Required: `DeleteInvitationByIdQuery` is a *blanket* impl over `Service<&DeleteInvitationParams>`, so a second SQL behavior on the same repository needs a distinct params type to avoid an overlapping impl.
2. **Postgres**: new module `oxidauth-postgres/src/invitations/delete_valid_invitation_by_id/` mirroring the existing one:
   ```sql
   DELETE FROM invitations
   WHERE id = $1 AND expires_at > NOW()
   RETURNING *
   ```
   impl `Service<&ClaimInvitationParams>` for `PgInvitationRepository`, `fetch_one(&self.db.write_pool())` (must stay on the write pool; it mutates).
3. **Repository**: `oxidauth-repository/src/invitations/delete_valid_invitation_by_id.rs` with `trait DeleteValidInvitationByIdQuery: for<'a> Service<&'a ClaimInvitationParams, Response = Invitation, Error = BoxedError>` + blanket impl, copying `delete_invitation_by_id.rs`.
4. **Service**: in `AcceptInvitationUseCase`, swap the generic bound/field from `DeleteInvitationByIdQuery` to `DeleteValidInvitationByIdQuery` and build `ClaimInvitationParams`. No other logic changes. Expired or unknown id → `sqlx::Error::RowNotFound` propagates, side effects (authority, update) never start — same shape as today's missing-row path.
5. **API/wiring**: `oxidauth-api/src/provider/services.rs:715` passes the same `PgInvitationRepository` struct; no change beyond re-export/import resolution. HTTP contract unchanged (error still → 400 envelope). Deliberately do not distinguish "expired" from "missing": on an unauthenticated endpoint that distinction is an existence oracle. If product later demands a distinct message, add a read-only `SELECT … expires_at` fallback **only on the RowNotFound path**.
6. **Leave `delete_invitation_by_id_query.sql` unfiltered** so admin `DELETE` can still revoke/delete expired invitations.
7. **Same-change hardening (small, no API change):** in `CreateInvitationUseCase::create_invitation`, reject `Some(expires_at <= now)` (and keep the 7-day default). Configurable-TTL work (TODO pivotal 186949366) stays out of scope.
8. **Cleanup/monitoring for expired rows:** with the fix, rejected expired rows still linger (the claim deletes nothing). There is no scheduler in the repo (`oxidauth-api` has no `tokio::spawn`/interval/cron — verified by grep; `bin/` is only deploy/test shell scripts; `oxidauth-cli` is still the `add(2,2)` template crate). Recommend, in order of cheapness: (a) document ops cleanup `DELETE FROM invitations WHERE expires_at < NOW() - interval '30 days'` via pg_cron/external cron — zero code; (b) fold it into a future `oxidauth-cli` maintenance command; (c) only if ops wants a metric, take the service-side variant (below) which can count expiries. No new migration or schema change is required for any of this.
9. **Alternative rejected:** refresh-token-style service-side check (`exchange_refresh_token.rs:117` pattern): `find_invitation_by_id` → compare `expires_at` in Rust → delete. Needs two queries, has a boundary TOCTOU (row expires between SELECT and DELETE — still safe against double-accept since only one DELETE wins, but expiry is enforced approximately), and duplicates the clock source. Rejected outright: adding the filter to the existing shared SQL (breaks admin delete, see Analysis).

**Pinned-test handling (flip when fixing):**
- `oxidauth-services/src/invitations/accept_invitation.rs:383–400` — delete the `BUG(pinned)` comment (:385–389); rewrite `expired_invitation_is_still_accepted` as `expired_invitation_is_rejected_without_side_effects`: mock claim query returns `RowNotFound` for an expired invitation (mimicking the SQL filter), assert the error and `steps == []`-after-claim / `user_authorities` and `updates` empty (mirror the structure of the missing-invitation test at :402–419, including no-delete/no-authority gating). The test mock (`MockDeleteInvitation`) must be ported to the new `ClaimInvitationParams`/trait.
- Do **not** touch the neighboring `BUG(pinned)` markers: `oxidauth-postgres/src/invitations/insert_invitation/mod.rs:101` belongs to SEC-13; `oxidauth-rs` client markers belong to CLI-6. The `RowNotFound` debug-string pin machinery in `oxidauth-kernel/src/error.rs` tests is for other endpoints' hurl and stays.
- Keep green as-is (they encode constraints this fix must respect): `missing_invitation_surfaces_…` (:402–419), the create-side 7-day-default bracket pin (`create_invitation.rs` omitted-expiry pin), `oxidauth-postgres` `select_invitation_by_id` / `delete_invitation_by_id` tests (both seed `expires_at = 2030-01-01`, so the untouched shared SQL keeps passing).

**Compat/migration:** no schema migration; no API shape change. Behavior change is exactly "accept after `expires_at` now fails 400 instead of succeeding" — the point of the ticket. Deploy note: invitations created before the fix with past `expires_at` stop working immediately; operators may want to re-issue pending invites. Old clients (`oxidauth-rs` `accept_invitation`) need no change; they already surface the error envelope.

## Verification

1. New postgres integration tests in `oxidauth-postgres/src/invitations/delete_valid_invitation_by_id/mod.rs` (follow `select_invitation_by_id` fixtures): future-expiry row → claim succeeds, returns the row, `COUNT(*)` afterwards 0; past-expiry row → `downcast_ref::<sqlx::Error>()` is `RowNotFound` and the row still exists (reject must not consume it); unknown id → `RowNotFound`. Plus one regression test on the *existing* unfiltered delete: it still deletes an expired row (guards the admin-delete decision in step 6).
2. `set -a && source .env && set +a && export MIGRATIONS_ENABLED=true && cargo test -p oxidauth-postgres -- invitations::` (live docker postgres, project convention from missing-tests.md).
3. `cargo test -p oxidauth-services -- invitations::` — flipped pin passes, the other four accept tests and create/find/delete tests unchanged.
4. End-to-end smoke against the real server (`docker-compose up -d postgres && cargo run -p oxidauth-api` or the repo's `bin/` scripts): with an admin JWT — `POST /v1/invitations` with `expires_at` one minute in the past → create (or, after hardening step 7, a 400), wait, then anonymous `POST /v1/invitations/{id}` with a valid accept body → 400 with `$.errors[0].debug == "RowNotFound"`, and `SELECT` confirms the row survives; repeat with the default expiry → 201/200 success and the row is consumed. Confirm admin `DELETE` on the expired row returns 200 (not 400).
5. `grep -rn 'BUG(pinned)' src/oxidauth --include='*.rs' | grep -i invitation` → the accept_invitation marker is gone; SEC-13 and CLI-6 markers remain.
