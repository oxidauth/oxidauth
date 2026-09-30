# OXA-000054 — user_permission_grants has no delete-by-grantee method (uniform grant-repo gap)

**Original ID:** N-4 · **Severity:** n/a · **Type:** note · **Status:** done
**Review tier:** Tier 1 (docs-only), ranked item 7/9 · IMPLEMENTED — TESTING.md §3 recipe landed, folklore-free (verified 2026-09-30 review pass)


## Locations

- `src/oxidauth/oxidauth-postgres/src/user_permission_grants/` — full repo surface: `insert_user_permission_grant/`, `select_user_permission_grants_by_user_id/`, `delete_user_permission_grant/` (+ shared row/struct defs in `mod.rs:12-74`). That is all three; there is no fourth module.
- `src/oxidauth/oxidauth-postgres/src/user_permission_grants/delete_user_permission_grant/mod.rs:22-28` — the only delete: binds `params.user_id` **and** `params.permission_id`, `DELETE … RETURNING *` + `fetch_one`, `Response = UserPermissionGrant` (single).
- `src/oxidauth/oxidauth-postgres/src/user_permission_grants/delete_user_permission_grant/delete_user_permission_grant.sql` — `DELETE FROM user_permission_grants WHERE user_id = $1 AND permission_id = $2 RETURNING *` (pair-scoped).
- Trait mirrors: `src/oxidauth/oxidauth-repository/src/user_permission_grants/{insert_user_permission_grant,select_user_permission_grants_by_user_id,delete_user_permission_grant}.rs` — exactly three query traits (`InsertUserPermissionGrantQuery`, `ListUserPermissionGrantsByUserIdQuery`, `DeleteUserPermissionGrantQuery`).
- DDL: `src/oxidauth/oxidauth-postgres/migrations/20221020012656_create_user_permission_grants.sql` — `PRIMARY KEY(user_id, permission_id)` (no id column, no grantor column), both FKs `ON DELETE CASCADE`, plus `user_permission_grants_user_id_idx` and `user_permission_grants_permission_id_idx`.
- Sibling grant repos, same 3-module shape: `oxidauth-postgres/src/user_role_grants/`, `role_permission_grants/`, `role_role_grants/` — deletes are pair-scoped there too (e.g. `role_permission_grants/delete_role_permission_grant/delete_role_permission_grant.sql`: `WHERE role_id = $1 AND permission_id = $2`).
- By-grantee precedent (cautionary): `oxidauth-postgres/src/refresh_tokens/delete_refresh_token_by_user_id/mod.rs:20` (`fetch_one`, `Response = RefreshToken`) — pinned-buggy shape, see OXA-000023.
- Consumers today: `oxidauth-api/src/server/api/v1/users/permissions/mod.rs:15-17` (routes: `GET /`, `POST /{permission}`, `DELETE /{permission}` — no bulk `DELETE /`), `oxidauth-rs/src/client/users/permissions/` (create / pair-delete / list-by-user-id wrappers only), `oxidauth-services/src/user_permission_grants/` (three use cases; delete resolves the permission string then deletes the pair), `oxidauth-postgres/src/auth/tree/mod.rs:12,78` (consumes `select_user_permission_grants_by_user_id_query`).

## Problem

Register line N-4 (`BUGS_AND_NOTES.md:86`): `user_permission_grants` has no delete-by-grantee counterpart; audit gaps A3.6/A6.6 were closed by testing `auth/tree` instead because the methods themselves don't exist.

Verified state (all four claims hold):

1. **The table's repo exposes exactly three operations**: insert (pair), select-by-user_id (joined `Vec<UserPermission>`), delete-by-pair. There is **no delete-by-id** — impossible by schema, the table has no id column (`PRIMARY KEY(user_id, permission_id)`); **no delete-by-grantor** — no grantor column exists; and **delete-by-grantee (`WHERE user_id = $1` alone) is absent**, confirmed by directory census and by `grep` for any `delete_*_by_user_id` / grantee-pattern delete under the grant modules (zero hits outside the refresh_tokens crate module).
2. **The gap is uniform, not user_permission_grants-specific**: none of the four grant tables has a grantee-wide delete. `user_role_grants` has **no** delete-by-user (only the pair delete); `role_permission_grants` has **no** delete-by-role (verified); `role_role_grants` has no delete-by-parent. The register's "counterpart" framing implies a sibling has one — it does not.
3. **No consumer needs one today**: no HTTP route bulk-deletes grants (`DELETE /v1/users/{user_id}/permissions` is not a route; only `/v1/users/{user_id}` and `/v1/users/{user_id}/permissions/{permission}` exist, `users/mod.rs:29,35`); no `oxidauth-rs` wrapper exists or implies one; `bin/` holds only shell scripts (the "CLI" ticket family 000027-000036 is the `oxidauth-rs` client surface — no bulk-grant-delete there). User-wide grant removal is handled by schema `ON DELETE CASCADE` when the user row is deleted (behavior registered under OXA-000020, including its silent-cascade objection).
4. **Audit closure is real but the item numbering is not recoverable from the repo**: `auth/tree` tests exist and are green-marked (`oxidauth-postgres/src/auth/tree/mod.rs:124+`, five tests; `missing-tests.md` A3 done, and A6's twelve rows all done cover exactly the twelve methods that exist). The original sub-IDs "A3.6/A6.6" appear nowhere else in the repository — `BUGS_AND_NOTES.md:86` is the sole record of what those sub-items asked for. Marking that mapping **[unverifiable]**; the substantive claim (methods don't exist, nothing tests them, nothing calls them) is fully verified.

`BUG(pinned)` grep: no pins inside `user_permission_grants/`; nearby pins are `user_role_grants/select_user_role_grants_by_user_id/mod.rs:131` (timestamp mapping, OXA-000018) and `role_role_grants/insert_role_role_grant/mod.rs:75,96` (cycle acceptance, OXA-000001). Nothing pins the *absence* of a grantee delete — unsurprising, a non-existent method cannot be tested.

## Analysis

The note is **accurate but describes a coherent design, not an oversight**. The grant repos were deliberately built around the schema:

- Pair-scoped delete mirrors the `PRIMARY KEY(user_id, permission_id)`: every grantee-wide bulk removal the product performs today goes through FK cascade (`DELETE FROM users` → grants vanish, `20221020012656:9-10`), which is already registered with its own defect ticket (OXA-000020 — silent, unlogged, non-idempotent cascade). An explicit grantee-delete would *not* change that cascade behavior; it would only serve flows the product does not have (e.g. "wipe all direct grants without deleting the user", an audited bulk-revoke, a user-reset feature).
- The single by-grantee-delete precedent in the codebase (`refresh_tokens/delete_refresh_token_by_user_id`) is itself a liability: `RETURNING *` + `fetch_one` loses the deleted count and errors on zero rows (OXA-000023), and its kernel `ServiceTrait` scaffolding is dead code — the sole real caller (`oxidauth-services/src/auth/strategies/username_password/forgot_password.rs:14`) consumes the repository query trait directly, and `grep -r DeleteRefreshTokenByUserIdService` hits only the definition file. Adding grant deletes with no caller invites cloning that dead scaffolding.
- A correct implementation is cheap and fully conventional (recipe below), but every layer above the repo (kernel service trait, DI wiring in `provider/services.rs`, HTTP handler, http req/res, SDK wrapper, hurl) would be speculative surface with no route to put it on. There is also a symmetry tax: shipping `user_permission_grants`-only would leave the same gap in its three siblings (delete-by-role for `role_permission_grants`, delete-by-user for `user_role_grants`, delete-by-parent for `role_role_grants`) — a half-uniform API is worse than the current uniformly-pair-scoped one.
- One adjacent bulk variant worth naming: **delete-by-permission** (`WHERE permission_id = $1`) is equally absent — the third conceivable bulk direction — reinforcing "3 methods by design" rather than a forgotten sibling of an existing bulk family.

**Disposition: KEEP-until-needed.** Not FIX NOW (no consumer, no route, and the cascade path already covers the only user-wide removal the product performs); not a pure KEEP-doc-only either, because the note's "counterpart" phrasing will keep misleading auditors into thinking one sibling *does* have grantee delete — the documentation change must state the uniform pair-scoped convention plus the ready-to-execute recipe. When a product slice needs bulk revoke (audited revoke-all, user reset, or the OXA-000020 cascade-visibility fix), execute the recipe below as a repo-layer-only slice across all four grant tables.

## Impact

- **Today: zero runtime impact.** No route, SDK method, service, or seeding flow requires grantee-wide grant delete; user removal cascades at the schema level. No data-integrity or security consequence — a missing method cannot be called, and the existing pair delete is fully tested (missing-pair error pinned in `delete_user_permission_grant/mod.rs:77-105`).
- **Documentation rot: this is the actual cost.** The note as written reads like an asymmetry between siblings; auditors re-deriving the gap waste cycles (this ticket's §Problem is that re-derivation). The untraceable A3.6/A6.6 IDs make the audit trail circular.
- **Latent cost if/when needed:** a bulk-revoke feature built on the existing pair delete would issue N round-trips per user inside whatever tx (or none) the caller manages, and would misreport partial state. The recipe is a same-day slice, so the note never threatens a deadline — it only needs to be findable, which §Proposed resolution ensures.

## Proposed resolution

**KEEP-until-needed.** Retiring the note = documentation + cross-reference, no code:

1. **Module doc in `oxidauth-postgres/src/user_permission_grants/mod.rs`** (and a one-line pointer in the three sibling `mod.rs` files): state the intentional surface — "insert / select-by-grantee / delete-by-pair; no grantee-wide delete by design: bulk removal is FK-cascade (`20221020012656:9-10`), explicit bulk revoke is deferred to OXA-000054's recipe; uniform across all four grant tables."
2. **Rewrite `BUGS_AND_NOTES.md:86`** to drop the phantom "counterpart" framing and the unrecoverable A3.6/A6.6 IDs (they exist nowhere in the repo): point at this ticket and record "pair-scoped deletes everywhere; recipe on file."
3. **The pre-approved implementation recipe** (execute only when a consumer exists; repo-layer slice — do *not* clone the dead kernel `ServiceTrait` scaffolding):
   - Kernel query struct: `oxidauth-kernel/src/user_permission_grants/delete_user_permission_grants_by_user_id.rs` — `pub struct DeleteUserPermissionGrantsByUserId { pub user_id: Uuid }` + `mod.rs` export (mirror `refresh_tokens/delete_refresh_token_by_user_id.rs` struct, skip its service trait).
   - Repo trait: `oxidauth-repository/src/user_permission_grants/delete_user_permission_grants_by_user_id.rs` — `DeleteUserPermissionGrantsByUserIdQuery` + blanket impl + error struct, same shape as `delete_user_permission_grant.rs` (trait file on file above).
   - Pg impl: `oxidauth-postgres/src/user_permission_grants/delete_user_permission_grants_by_user_id/{mod.rs,delete_user_permission_grants_by_user_id.sql}` — SQL `DELETE FROM user_permission_grants WHERE user_id = $1 RETURNING *` (hits `user_permission_grants_user_id_idx`); **`fetch_all` → `Response = Vec<UserPermissionGrant>` with empty-vec-on-nothing**, i.e. the OXA-000023 fix shape, never the `fetch_one` precedent.
   - Symmetry: same trio for `role_permission_grants/delete_role_permission_grants_by_role_id` (verified missing), and — to keep the four-table API uniform — `user_role_grants/delete_user_role_grants_by_user_id` + `role_role_grants/delete_role_role_grants_by_parent_id`.
   - Tests per **`docs/TESTING.md` §3** (the corrected test conventions; it supersedes the T-6 register folklore): `#[sqlx::test(migrator = "crate::MIGRATOR")]`, `Database::from_pool(pool.clone())` — the pool is taken BY VALUE — via the canonical per-module `fn repo(pool: &PgPool)` helper (`Database::from_pool(&pool)` is disproved folklore and does not compile); seed via `crate::test_fixtures` (`seed_user`, `seed_permission`, `assert_sql_state`; coordinate the `test_fixtures` dedupe state with OXA-000053 at execution time). Minimum four tests: deletes all of one user's grants and returns them; leaves other users' rows intact; empty vec (not `RowNotFound`) when the grantee has none; `permissions` rows survive the delete (anti-cascade proof in the app layer the schema won't give you).
   - Only then, if a route demands it: service use case + DI block next to `oxidauth-api/src/provider/services.rs:275-305` + handler under `server/api/v1/users/permissions/` (`DELETE /` route) + http req/res + `oxidauth-rs` wrapper + hurl case, following `docs/migration-plan/09-services-layer.md:156-158` conventions.

## Verification

Current-state confirmation (read-only, safe today):

- Grant repo surface = 3 modules each, no bulk delete: `ls src/oxidauth/oxidauth-postgres/src/{user_permission_grants,user_role_grants,role_permission_grants,role_role_grants}` — every `delete_*` dir is the pair-scoped one.
- Grantee delete truly absent: `grep -rn "delete_user_permission_grants_by_user_id\|DeleteUserPermissionGrantsByUserId\|delete_role_permission_grants_by_role_id" src/` → zero hits; `grep -rn "WHERE user_id = \$1" src/oxidauth/oxidauth-postgres/src/*/delete*/*.sql` → refresh_tokens only.
- No bulk route / wrapper: `grep -n "route(" src/oxidauth/oxidauth-api/src/server/api/v1/users/permissions/mod.rs` (no `delete` on `"/"`) and `ls src/oxidauth/oxidauth-rs/src/client/users/permissions/` (three pair-scoped wrappers).
- Cascade coverage of the user-wide case: `grep -n "ON DELETE CASCADE" src/oxidauth/oxidauth-postgres/migrations/20221020012656_create_user_permission_grants.sql` (both FKs).
- No nearby pins: `grep -rn "BUG(pinned)" src/oxidauth/oxidauth-postgres/src/user_permission_grants` → zero hits.

After the doc-only retirement lands: `grep -rn "A3.6\|counterpart" BUGS_AND_NOTES.md` shows only the rewritten N-4 line pointing here.

If the recipe is ever implemented: `set -a && source .env && set +a && cargo test -p oxidauth-postgres -- user_permission_grants::` green (live docker postgres per T-6), the new module's four tests enumerated in the run log, and the symmetry trio present in all four grant repo dirs.

Cross-references: OXA-000020 (silent cascade — the only user-wide removal path today, same fix family if bulk revoke becomes audited), OXA-000023 (delete-count-lost — the anti-pattern this recipe must not clone), OXA-000024 (honest `by_user_id` naming for the new query), OXA-000018 (grant-timestamp mapping in the joined `UserPermission` type), OXA-000039 (auth/tree consumption, the surface the audit tested instead); notes N-3 (test_fixtures dedupe), T-6 (pg test conventions).

## Decision (2026-09-29) — ACCEPTED (KEEP-until-needed, doc-only), scheduled; no implementation started
- **Review tier: Tier 1 (docs-only), item 7/9** (pulled from original list position 6; review order differed).
- Approved as three concrete doc edits, zero code:
  1. Module doc on `oxidauth-postgres/src/user_permission_grants/mod.rs`: intentional surface (insert / select-by-grantee / delete-by-pair); composite PK makes the pair the only grant identity; user-wide removal = FK CASCADE (`20221020012656:9-10`); bulk revoke deferred with recipe; uniform across all four grant tables.
  2. One-line pointer doc on the three sibling `mod.rs` (`user_role_grants`, `role_permission_grants`, `role_role_grants`).
  3. `BUGS_AND_NOTES.md:86` (N-4) rewritten: drops phantom "counterpart" framing and unrecoverable A3.6/A6.6 IDs; records "pair-scoped everywhere; recipe on file at OXA-000054".
- NOT fixing (confirmed): no `delete_*_by_grantee` modules, no kernel/repo/HTTP/SDK speculative layers — composite PK + cascade cover today's flows; the `refresh_tokens/delete_refresh_token_by_user_id` fetch_one/dead-scaffolding precedent is the anti-pattern to avoid.
- Recorded constraint on the step-3 recipe: at execution time its test guidance re-points to `docs/TESTING.md` §3 (from OXA-000066/68) — the embedded T-6 citation's `Database::from_pool(&pool)` is disproved folklore; use `Database::from_pool(pool.clone())` via the `fn repo(pool: &PgPool)` helper. Recipe executes across all four tables (symmetry), `fetch_all → Vec`, empty-vec-not-RowNotFound, and only when a real consumer exists.
- Cross-refs stand: OXA-000020 (cascade visibility), 23 (anti-pattern), 53 (fixture dedup state at execution), 24 (naming honesty).
- Acceptance: `grep -rn "A3.6\|counterpart" BUGS_AND_NOTES.md` shows only the rewritten N-4 line; five module docs present. Doc-only — nothing to build.
