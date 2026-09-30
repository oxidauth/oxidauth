# OXA-000018 — `From<PgUserRole>` overwrites role timestamps with the grant's; the SQL's real role timestamps are discarded

**Original ID:** DATA-5 · **Severity:** P2 · **Type:** bug · **Status:** open
**Review tier:** Tier 2 (mechanical 1-3 line code fix) — ranked #15 of 27 · reviewed 2026-09-29 · DEFERRED (owner decision; claims verified, fix plan sound — revisit later)


## Locations

All paths relative to `src/oxidauth/` unless noted. Verified against the working tree 2026-09-29.

The defect:

- `oxidauth-postgres/src/user_role_grants/mod.rs:53-69` — `impl From<PgUserRole> for UserRole`. The wrong lines are `:59-60`, inside the `role:` struct literal: `created_at: value.created_at` / `updated_at: value.updated_at`, where `value.created_at`/`value.updated_at` are the **`user_role_grants` join columns** (struct fields `:46-47`). The role's own timestamps sit unused in `value.role_created_at` / `value.role_updated_at` (`PgUserRole` fields `:49-50`), which `sqlx::FromRow` populates from the SQL but nothing ever reads. The grant half of the mapping (`:62-67`) is correct.
- `oxidauth-postgres/src/user_role_grants/select_user_role_grants_by_user_id/select_user_role_grants_by_user_id_query.sql:7-8` — the query **does** fetch the real values: `roles.created_at AS role_created_at, roles.updated_at AS role_updated_at`. Only the Rust mapping throws them away.
- `oxidauth-postgres/src/user_role_grants/select_user_role_grants_by_user_id/mod.rs:25` — the single conversion site, `.map(Into::into)` in `Service::call`, is the only consumer of the broken `From` impl (grep for `PgUserRole\b`: struct definition, this query, and the pin comment — nowhere else).

The pin (where the register's `:131` actually points):

- `oxidauth-postgres/src/user_role_grants/select_user_role_grants_by_user_id/mod.rs:69-137` — test `it_should_join_the_role_and_pin_the_grant_timestamp_mapping`; `BUG(pinned)` comment at `:131-134`; pinned assertions at `:135` (`assert_eq!(granted.role.created_at, granted.grant.created_at)`) and `:136` (`assert_ne!(granted.role.created_at, role_ts)`, where `role_ts` is the backdated row's real `created_at` read straight from `roles` at `:120-125`). The fixture deliberately backdates the role to 2000-01-01 (`:74-84`) so the two timestamp pairs are distinguishable.

Reference (the correct sibling — this defect is **not** shared):

- `oxidauth-postgres/src/role_role_grants/mod.rs:65-81` — `From<PgRoleRoleGrantDetail> for RoleRoleGrantDetail` maps exactly the way `PgUserRole` should: `role.created_at = pg.role_created_at` (`:71-72`), `grant.created_at = pg.created_at` (`:77-78`). Its query likewise aliases `roles.created_at AS role_created_at` (`select_role_role_grants_by_parent_id.sql:7-8`). Same shape, same intent, correct code — `From<PgUserRole>` is the lone outlier. `From<PgRole> for Role` (`oxidauth-postgres/src/roles/mod.rs:32-40`) is trivially correct (no join).

**Register drift (confirmed, matches the known pattern):** the register cites `oxidauth-postgres/src/user_role_grants/select_user_role_grants_by_user_id/:131`. That path/line is the first line of the `BUG(pinned)` comment in that module's test block (`mod.rs:131`), not the defect. The defect lives one directory up, in the **parent** `user_role_grants/mod.rs:59-60` — the register's path points at the child *query* module, while the `From` impl is on the shared parent. The substantive claim is exactly right.

## Problem

Every read of "which roles does user X have" returns roles whose `created_at`/`updated_at` are the **grant row's** timestamps, not the role row's. Mechanically:

1. SQL joins `user_role_grants` to `roles` and selects all four timestamps (`select_user_role_grants_by_user_id_query.sql:2-8`); `PgUserRole` decodes all six columns fine (`FromRow` ignores nothing here — every field has a matching column).
2. `From<PgUserRole> for UserRole` copies the grant's `created_at`/`updated_at` into **both** `UserRole.role.*` (`mod.rs:59-60`) and `UserRole.grant.*` (`:65-66`), and never touches `role_created_at`/`role_updated_at`. The decoded role timestamps are silently dropped on every row.
3. The result is duplicated, wrong: for one row, `role.created_at == grant.created_at` and `role.updated_at == grant.updated_at`, by construction, regardless of what `roles` says.

The wrong value isn't merely imprecise — it is a *different fact*. A grant can only be created after the role exists, so `grant.created_at >= role.created_at` always; and if the role is ever renamed (`updated_at = NOW()` on `roles`, e.g. `update_role`), `role.updated_at > grant.updated_at` is possible. The listed role timestamps therefore drift arbitrarily (typically years late) from the true ones, and **differ per grant**: granting the same role to 1,000 users makes the same `Role.id` report 1,000 different `created_at` values.

**Endpoint-to-endend contradiction.** Only the list path goes through the broken `From`:

- `POST /users/{user_id}/roles/{role_id}` and `DELETE` /users/{user_id}/roles/{role_id} build `UserRole` in the use case from `FindRoleById` + the grant query (`oxidauth-services/src/user_role_grants/create_user_role_grant.rs:62-77`, `delete_user_role_grant.rs:62-77`) — those return the **true** role timestamps (`PgRole` → `Role`, `oxidauth-postgres/src/roles/mod.rs:32-40`).
- `GET /api/v1/users/{user_id}/roles` serializes `ListUserRoleGrantsByUserIdRes { user_role_grants: Vec<UserRole> }` (`oxidauth-http/src/users/roles/list_user_roles_by_user_id.rs:10-11`; handler `oxidauth-api/src/server/api/v1/users/roles/list_user_roles_by_user_id.rs:57`; route nested at `oxidauth-api/src/server/api/v1/users/mod.rs:36`, gated on `PERMISSION = "oxidauth:users:manage"`, `users/mod.rs:20`).

So a client that grants a role and lists it sees `role.created_at` **change** (grant time ≠ role time), and `GET /roles` / `GET /roles/{id}` contradict `GET /users/{id}/roles` about the same object. The SDK re-exports the same typed response (`oxidauth-rs/src/client/users/roles/list_user_roles_by_user_id.rs:3`), so `oxidauth-rs` users get the corrupted fields in typed form; `oxidauth-rs` mock/contract tests are route-shape tests over canned JSON (`oxidauth-rs/src/client/users/contract.rs:282-291, 386-391`) and pin nothing.

## Analysis

**Mechanism.** Two-field typo-shaped bug in a hand-written `From` join-mapping. The struct author clearly intended the alias pattern — they added `role_created_at`/`role_updated_at` to `PgUserRole` *and* aliased them in the SQL — then copied `value.created_at`/`value.updated_at` into the role literal out of order/habit. `role_role_grants` (written with `pg.` naming and correct mapping) shows the intended design; whichever came first, `PgUserRole`'s impl deviated from it. Nothing catches this class of error: sqlx `FromRow` is name-based and cannot know that `Role.created_at` should come from `role_created_at`, the crate uses runtime `query_as(include_str!)` (no `query!` compile-time checking), and serde just serializes whatever it's given.

**Edge cases / related paths.**

- **Silent + per-row:** no error, no log, no panic — the only signal is client-visible inconsistency. Any consumer doing "has this role definition changed since it was granted?" (`role.updated_at > grant.updated_at`) sees a constant `false` from the list endpoint, because they're equal by construction (`mod.rs:59-60` vs `:65-66`).
- **Consumers of the poisoned fields are few but real.** The only in-repo consumer of the list service, bootstrap's `add_admin_role_to_admin_user` (`oxidauth-services/src/bootstrap/mod.rs:504-510`), matches on `grant.role.id` only — unaffected. Authorization/entitlement paths never read these timestamps. Blast radius is therefore the **wire contract**: API/SDK/CLI consumers of `GET /users/{id}/roles` JSON. (Grep across `oxidauth-import-export` and `oxidauth-cli`: no `UserRole` references at all.)
- **hurl/SDK coverage asserts no timestamps on this endpoint.** `hurl/tests/user_roles.hurl` checks ids/names/counts only (`:49-55, 63-64, 71-72`); the `role.created_at exists` asserts in `hurl/tests/roles.hurl:49-50` are on the create-role endpoint, which is correct code. Nothing else needs updating outside the pin.
- **Exactly one related `BUG(pinned)` marker:** grep of `BUG(pinned)` for this item returns only `select_user_role_grants_by_user_id/mod.rs:131`. No service-level, SDK, or hurl pins exist.
- **The reverse mapping hazard the pin cannot catch:** the current pin would pass equally for an impl that swaps `role_created_at`↔`role_updated_at` (both are backdated to the same instant, `mod.rs:77-78`), and for an impl that mapped role's timestamps into `grant.*`. The flip should use *distinct* instants per field (see Verification).
- **Grant-level timestamps need no product decision** — the register's open question is already answered by the codebase: `UserRole.grant.created_at`/`updated_at` is the grant's surfacing (`oxidauth-kernel/src/user_role_grants/mod.rs:17-22`, serialized in the response as `grant.*`), and `role_role_grants` exposes the identical `{ role, grant }` split (`RoleRoleGrantDetail`). The fix is to stop leaking grant values into `role.*`, not to invent a new field.

**Who is affected.** Any API/SDK consumer reading `payload.user_role_grants[*].role.created_at|updated_at` — e.g. UIs rendering "role created", sync/audit tooling comparing role versions across endpoints, anyone diffing `GET /roles` vs `GET /users/{id}/roles`. Not affected: permission evaluation, bootstrap idempotency (id-compare), create/delete responses. P2 is right: persistent wrong data on a public read path, no authz impact, trivially fixable.

## Impact

- **Correctness:** `GET /users/{id}/roles` misreports both role timestamps for every grant row; the same role reports different `created_at` per grantee and contradicts every other role endpoint.
- **Compatibility:** the response **shape is unchanged** — same fields, same types; only values become truthful. No DB migration, no schema change, no SQL change, no wire-format change. Consumers that (mis)relied on `role.created_at == grant.created_at` from the list endpoint would notice — that reliance was relying on the bug; call it out in the changelog.
- **Test contract:** the pinned test asserts the broken equality and must flip in the same commit; it cannot coexist with the fix.

## Proposed resolution

**Step 1 — the two-line fix** in `oxidauth-postgres/src/user_role_grants/mod.rs:56-61`, mirroring the `role_role_grants` reference impl (`role_role_grants/mod.rs:68-73`):

```rust
role: Role {
    id: value.role_id,
    name: value.name,
    created_at: value.role_created_at,   // was value.created_at  (grant's!)
    updated_at: value.role_updated_at,   // was value.updated_at  (grant's!)
},
```

Keep the `grant:` half (`:62-67`) exactly as is — `value.created_at`/`value.updated_at` *are* the grant's and already surface correctly there. No change to `PgUserRole`, the SQL, the trait, kernel types, DTOs, or the SDK: every layer above already carries both timestamp pairs.

**Step 2 — flip the pin** in `select_user_role_grants_by_user_id/mod.rs` (the only marker for this item):

- Delete the `BUG(pinned)` comment (`:131-134`) and rename the test (`:70`) — `..._pin_the_grant_timestamp_mapping` describes the defect, not the behavior.
- Invert the two assertions against the already-seeded backdated fixture: `:135` `assert_eq!(granted.role.created_at, granted.grant.created_at)` → `assert_ne!`, and `:136` `assert_ne!(granted.role.created_at, role_ts)` → `assert_eq!(granted.role.created_at, role_ts)` (the fixture's 2000-01-01 row, `role_ts` read at `:120-125`).
- Strengthen while fixing the blind spots above: backdate `created_at` and `updated_at` to **different** instants in the seed (`:77-78` currently uses one value for both), fetch both from `roles`, and add the `updated_at` legs (`assert_eq!(granted.role.updated_at, role_updated_ts)` and `assert_ne!(granted.role.updated_at, granted.grant.updated_at)`) — this is what makes the pin catch a created/updated swap or a mapping into `grant.*`. The existing structural asserts (`:113-118`) still hold; keep them.

**Step 3 — optional regression lock at the HTTP boundary** (cheap, recommended): in `hurl/tests/user_roles.hurl`, capture `$.payload.role.created_at` from the `POST /roles` step (`:28-37`) and assert `$.payload.user_role_grants[0].role.created_at` equals it in the list step (`:57-64`). Today's hurl has no timestamp asserts on this endpoint, so nothing breaks; this is the only automated proof the fix survives to the wire. No SDK change — contract tests are canned-JSON shape tests and stay as-is.

**No migration/compat work otherwise:** no SQL, no new fields, no versioned-API decision. One atomic commit: fix + pin flip (+ hurl leg).

## Verification

DB-backed suite is the proof surface (crate uses runtime `query_as`, so `cargo build` alone proves nothing here):

- `cargo test -p oxidauth-postgres --lib user_role_grants` (DB per `src/oxidauth/database_test.sh`; tests run against `crate::MIGRATOR` via `#[sqlx::test]`). Before the fix the current pinned test passes (broken equality asserted); after Step 1 alone it must **fail** on `:135/:136` — that failure is itself the repro. After Step 2 the flipped test passes: role timestamps == the backdated row (`< 1_000_000_000` epoch, already asserted at `:126-129`), grant timestamps == `now`-ish grant row, and with distinct seeds the `updated_at` legs pass too.
- Cross-endpoint truth check (manual or psql + curl): create role R, wait, grant R to a user, then compare `GET /api/v1/roles` (true `role.created_at`) vs `GET /api/v1/users/{user_id}/roles` — post-fix the two agree; on current code the list endpoint reports the grant instant. `sqlx::test` scratch or `psql -c "SELECT r.created_at, g.created_at FROM roles r JOIN user_role_grants g ON g.role_id = r.id WHERE r.id = '<r>'"` gives the same comparison at the source.
- `hurl -b hurl/tests/user_roles.hurl` (via `src/oxidauth/hurl.sh`) after Step 3, against a fresh DB, must pass — including the new `role.created_at` equality leg, which would fail on the unfixed mapping.
- `cargo test -p oxidauth-postgres --lib role_role_grants` unchanged as a control (the correct sibling must keep passing, proving the pattern, not the preference, was applied).

## Decision (2026-09-29) — DEFERRED; reviewed, not scheduled; no implementation started
- Reviewed jointly (scout re-verification + owner ruling 2026-09-29). Owner elected to defer; all substantive claims are CONFIRMED and the ticket stands as written for a future batch.
- Verified state carried forward: defect present at `oxidauth-postgres/src/user_role_grants/mod.rs:59-60` (grant timestamps copied into `role.*`); `PgUserRole.role_created_at`/`role_updated_at` (`:49-50`) decoded from the SQL aliases but never read; sole consumer `.map(Into::into)` at `select_user_role_grants_by_user_id/mod.rs:25`; correct sibling at `role_role_grants/mod.rs:68-80` is the reference; exactly one `BUG(pinned)` marker at `select_user_role_grants_by_user_id/mod.rs:131-134` pins the broken equality.
- Register precision noted for the eventual fix commit: DATA-5's cited path is the child query module (where the marker lives); the defect is the parent `From` impl — rewrite the line on retirement, don't just delete.
- Fix plan endorsed for when it is picked up: two-line `From` change → `value.role_created_at`/`value.role_updated_at`; flip + strengthen the pin (backdate `created_at`/`updated_at` to DISTINCT instants, add the `updated_at` assertion legs — the current tied fixture cannot catch a created/updated swap); optional hurl timestamp-equality leg on `user_roles.hurl` (currently asserts ids/counts only, so it can only add coverage). No SQL/schema/wire change; shape unchanged, values become truthful.
- Sequencing reminder for the future implementer: if OXA-000069's Service→Query migration has landed by then, the module's header will have changed but the `From` impl and the test assertions are untouched by that rewrite — coordinates above hold modulo that migration.
- **Adopted ownership from OXA-000053 review (2026-09-30):** the pin-fixture rewrite (`select_user_role_grants_by_user_id/mod.rs:69-137`) owns two raw `INSERT INTO user_role_grants (user_id, role_id)` copies (`:94`, `:100`). When this ticket is picked up, express both pairs as `test_fixtures::seed_user_role_grant(pool, user, role)` once OXA-000053(a) adds that export; the backdated-role INSERT (`:74-84`) stays inline — test-specific semantics by the `test_fixtures.rs:9-11` rule. If 000053(a) hasn't landed, add the export here.
