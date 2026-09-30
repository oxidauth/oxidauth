# OXA-000001 — Role grants accept cycles; permission-tree recursion stack-overflows

**Original ID:** SEC-1 · **Severity:** P1 · **Type:** bug · **Status:** open
**Review tier:** Tier 4 (moderate — multi-file recipes, pinned-test flips) · reviewed 2026-09-30 · DEFERRED (owner decision; claims verified intact at HEAD — one-shot permanent-DoS P1, 3-layer fix plan sound — revisit before any prod push)


## Locations

Register paths are crate-relative; the workspace root nests them under `src/oxidauth/`. Verified current locations:

| Role | Path | Key lines |
|---|---|---|
| Write path (DB) | `src/oxidauth/oxidauth-postgres/src/role_role_grants/insert_role_role_grant/mod.rs` | `call` 12–23; pinned tests at 74 (`BUG(pinned)` @75) and 95 (`BUG(pinned)` @96) |
| Write SQL | `src/oxidauth/oxidauth-postgres/src/role_role_grants/insert_role_role_grant/insert_role_role_grant.sql` | plain `INSERT … RETURNING *` |
| Schema | `src/oxidauth/oxidauth-postgres/migrations/20221020012709_create_role_role_grants.sql` | PK `(parent_id, child_id)` :7; FKs :9–10; indexes :13–14; no CHECK |
| Read path (write-use-case) | `src/oxidauth/oxidauth-services/src/role_role_grants/create_role_role_grant.rs` | 44–67 — existence-checks both roles, then inserts; no guard |
| HTTP entry | `src/oxidauth/oxidauth-api/src/server/api/v1/roles/roles/create_role_role_grant.rs` | gated on `oxidauth:roles:manage` (`roles/mod.rs:18`); route `POST /roles/{parent_id}/roles/{child_id}` (`roles/mod.rs:17`, nest at `roles/mod.rs:28`); service errors → 400 (handler line 62) |
| Recursion (crash site) | `src/oxidauth/oxidauth-postgres/src/auth/tree/mod.rs` | `role_permissions_as_tree` 91–121 (recursive call :105, no visited-set); `user_permissions_as_tree` :73 feeds it; repro test `it_should_recurse_forever_on_a_role_role_grant_cycle` 344–424 (`BUG(pinned)` @380) |
| Consumers of the tree | `oxidauth-services/src/auth/authenticate.rs:227`, `auth/register.rs`, `auth/authenticate_or_register.rs`, `refresh_tokens/exchange_refresh_token.rs:142`, `totp/validate.rs` | all with `PermissionSearch::User(…)`; `authenticate.rs:232` bakes the flattened list into JWT entitlements |
| Test helper | `src/oxidauth/oxidauth-postgres/src/test_fixtures.rs:91` | `assert_sql_state` for SQLSTATE assertions |

Latest migration (for naming the new one): `20250610213132_add_nbf_offset_to_authority_settings.sql`.

## Problem

Nothing in the stack — schema, repository insert, service use-case, HTTP handler — rejects a cyclic or self-referencing role→role grant.

- The table's only protection is the composite primary key `PRIMARY KEY(parent_id, child_id)`, which rejects **only the identical `(parent, child)` pair** (SQLSTATE 23505). There is no `CHECK (parent_id <> child_id)`, no trigger, no cycle detection anywhere (register and repo grep agree; the two `BUG(pinned)` tests prove acceptance end-to-end).
- `A→A` is accepted in **one** admin call (`POST /roles/{A}/roles/{A}`); `A→B` then `B→A` is accepted in **two**. The service layer (`create_role_role_grant.rs:44–67`) only checks both roles exist, then inserts verbatim.
- On the read side, `role_permissions_as_tree` (`auth/tree/mod.rs:91–121`) recurses over `select_role_role_grants_by_parent_id_query` children with **no visited-set and no depth limit**. A cycle makes it recurse unboundedly. The repro test documents observed behavior: on the default 2 MB test-thread stack the recursion SIGABRTs with a stack overflow in well under a second (`#[async_recursion]` boxing does not help — "each poll walks the whole nested-box future chain"); the test therefore runs the call on a dedicated 1 GB-stack thread so a 3 s `tokio::time::timeout` wins first (`mod.rs:385–393`).

The register line checks out exactly, except that it understates one point: a self-grant alone (one call) is already sufficient.

## Analysis

**Mechanism.** `role_permissions_as_tree` walks the grant graph depth-first, issuing per-node queries on one pooled connection and pushing a `RoleNode` per child. With edge `A→B` and edge `B→A`, the walk `A→B→A→B→…` never revisits a completed node — there is no notion of completion at all; each frame awaits its child before returning. The future for level *n+1* is polled from within level *n*'s poll, so the native stack grows per hop until the guard page fires. Rust stack overflow is `SIGSEGV`/`SIGABRT` — the **entire server process dies**, it is not a catchable panic.

**Who can reach it.**
- *Triggering:* any caller holding `oxidauth:roles:manage` (also bootstrap seeding, though seed data is acyclic). Insider, misconfigured integration, or stolen admin token.
- *Crashing:* every user whose authentication walks into the cycle. The tree runs inside `authenticate` (TotpSettings::Disabled branch, `authenticate.rs:225–238`), `register`, `authenticate_or_register`, `exchange_refresh_token`, and `totp/validate`, always as `PermissionSearch::User(user_id)`. Grant `A→B→A`, attach one user to `A`, and **every login and every refresh-token exchange for that user kills the process**. A cycle with no attached user is dormant — recursion starts only from a requested user/role root — until someone grants a role in the cycle to any user. `PermissionSearch::Role` is currently exercised only in tests; no HTTP route invokes the tree directly.
- *Blast radius:* process-wide abort takes down all in-flight sessions/requests of every tenant sharing the process, and the outage **persists** after restart — the cyclic rows are in the database, so the condition re-triggers on every login by an affected user until someone manually `DELETE`s grant rows. It is a one-grant, permanent, self-re-arming DoS.

**Guard placement nuances.**
- A SQL `CHECK` constraint can express only the self-loop case (`parent_id <> child_id`); Postgres has no constraint for transitive acyclicity, so longer cycles need an insert-time walk, a trigger, or the application layer.
- A guard in the repository/service does **not** cover raw SQL inserts — the existing tests themselves seed cycles via raw `INSERT INTO role_role_grants` (`auth/tree/mod.rs:164–184, 351–371`), so any operational SQL access bypasses app-level guards. This argues for schema-level enforcement as the primary line and the tree visited-set as defense-in-depth.
- **TOCTOU:** a check-then-insert ("walk ancestors of parent; reject if child is among them") races: concurrent `A→B` and `B→A` can both pass the check. Two inserts for the same pair are still serialized by the PK, but different pairs closing a cycle are not.
- The correct visited-set for the tree is a **path-local (ancestor) set**, not a global one: a global set would silently prune diamond-shaped DAGs (role `P` grants `X` and `Y`, both grant `D`), changing the tree/entitlement output and colliding with the separate dedup item SRV-3. Cutting only edges back into the current path preserves DAG semantics while guaranteeing termination. (Separately, diamonds cause exponential subtree duplication — related but out of scope here.)
- Deleting edges is safe and unchanged: FKs are `ON DELETE CASCADE`, so role deletion dissolves edges/cycles automatically.

**Pinned markers tied to this item** (`grep -rn 'BUG(pinned)' src --include='*.rs'`):
1. `oxidauth-postgres/src/role_role_grants/insert_role_role_grant/mod.rs:75` — `it_should_allow_a_self_referencing_grant_parent_eq_child` asserts the self-loop is stored.
2. `oxidauth-postgres/src/role_role_grants/insert_role_role_grant/mod.rs:96` — `it_should_allow_a_two_role_cycle` asserts both `A→B` and `B→A` succeed (`count_edges == 2`).
3. `oxidauth-postgres/src/auth/tree/mod.rs:380` — `it_should_recurse_forever_on_a_role_role_grant_cycle` (the timeout repro) asserts the tree call does **not** complete within 3 s.

`oxidauth-postgres/src/auth/tree/mod.rs:261` and `oxidauth-kernel/src/auth/tree/mod.rs:291` are `BUG(pinned)` markers in the same file(s) but belong to **SRV-3** (no dedup in the flatten) — do not flip them here.

## Impact

- **Availability (primary):** one or two admin-scoped HTTP calls turn every subsequent login/refresh/TOTP-validate for any user attached to a cyclic role chain into a process abort; survives restarts; recovery requires manual DB surgery. P1 as registered.
- **Correctness (secondary):** even a non-terminating-crash view of the same bug — a cycle silently *accepted* means the role graph is no longer a "hierarchy," so any future feature reasoning about inheritance (effective-permission previews, role deletion impact, UI tree renderers in `oxidauth-rs`/`oxidauth-http` payloads) inherits a corrupt invariant.
- **Affected parties:** every deploy allowing multiple holders of `oxidauth:roles:manage`; every end user of any authority sharing the crashed process. Unauthenticated users cannot trigger it directly, but the crash cost is paid by them, not the triggerer.
- No data corruption in `role_role_grants` itself; entitlements in already-issued JWTs are unaffected (they were flattened at mint time).

## Proposed resolution

Three layers; land them together in one PR (the tree guard makes the guardless window survivable while ops cleans pre-existing cycles).

1. **Migration — detect and clean existing cycles** (new file, e.g. `migrations/<ts>_role_role_grants_cycle_guard.sql`):
   - `DO $$ … $$` block that computes reachability with a recursive CTE over the current table and `RAISE EXCEPTION 'role_role_grants contains N cyclic grants: …'` listing offending edges (fail-closed). Do **not** auto-delete: choosing which closing edge to drop is a silent permission change. Provide the detection query and a copy-paste cleanup `DELETE` in the migration's header comment for operators.
   - Then `ALTER TABLE role_role_grants ADD CONSTRAINT role_role_grants_no_self_loop CHECK (parent_id <> child_id);` (existing test data contains no self-loops unless someone already made one; use `NOT VALID` + `VALIDATE CONSTRAINT` if a self-loop could exist and you want the migration non-blocking — it can't, once the cycle check above passed with empty result).
2. **Insert-time guard for longer cycles, race-safe** in `insert_role_role_grant` (`PgRoleRoleGrantRepository::call`), keeping the single-`Service` shape:
   - In one connection/transaction: `SELECT pg_advisory_xact_lock(<const>)` to serialize grant writes (grant rate is negligible; this closes the TOCTOU window without touching the API contract), then a recursive-CTE reachability probe — does `parent_id` reach `child_id` walking `role_role_grants` from `child_id`'s descendants? — and reject with a domain error (`"cycle: role X is already granted (transitively) to role Y"`) before the `INSERT`. A single statement `INSERT … SELECT … WHERE NOT EXISTS (WITH RECURSIVE …)` is an acceptable alternative *inside the same advisory lock*. No new `select_role_role_grants_by_child_id` query trait is needed if the probe is raw SQL here; if a repository trait method is preferred, add it under `oxidauth-repository::role_role_grants` mirroring `select_role_role_grants_by_parent_id`.
   - The existing handler already maps service errors to **HTTP 400** (`create_role_role_grant.rs:62`), so a rejected cycle reads as a clear client error; no API/product decision needed beyond wording. Decide to keep the guard in the postgres adapter (all runtime callers funnel through it; the service layer stays transport-agnostic).
3. **Defense-in-depth in the tree** (`auth/tree/mod.rs`): thread a path-local `&mut HashSet<Uuid>` (insert on entry, remove on exit) through `role_permissions_as_tree`; skip a child already on the current path (the graph is authoritative-clean post-guard; this only protects against raw-SQL rows and future regressions). Optional cheap belt: a max-depth constant (~64) returning an error. This also makes pre-existing cycles harmless between deploy and operator cleanup.

**Compat/migration notes:** no API-shape or wire-format changes; `RoleRoleGrant` DTO untouched. Existing DBs must pass the detection block before the CHECK lands — document the cleanup query in release notes. `#[sqlx::test]` runs the full `MIGRATOR`, so every postgres test pays the new migration (trivial cost). Clients (`oxidauth-rs`) need no change; a grant that used to 200 can now 400 — worth a changelog line since integrations may (incorrectly) have relied on cyclic grants as cycles-as-DAG-shorthands; flag in release notes.

**Pinned tests to flip** (per register: fix = flip assertion + delete marker):
- `it_should_allow_a_self_referencing_grant_parent_eq_child` → rename `it_should_reject_a_self_referencing_grant_parent_eq_child`; `expect_err` and `assert_sql_state(err, "23514")` (check_violation from the schema CHECK is the first thing that fires), `count_edges == 0`. Delete marker @75.
- `it_should_allow_a_two_role_cycle` → rename `it_should_reject_a_two_role_cycle`; `A→B` still succeeds; `B→A` `expect_err` asserting the guard error text mentions the cycle, `count_edges == 1`. Delete marker @96. (This one exercises the CTE guard + advisory lock, since the CHECK alone does not see 2-cycles.)
- `it_should_recurse_forever_on_a_role_role_grant_cycle` → rename `it_should_terminate_on_a_role_role_grant_cycle`; keep the **raw-SQL** cycle seeding (that's the point: schema guard does not cover raw inserts), delete the 1 GB-stack thread + 3 s timeout harness entirely, call the tree directly, and assert it completes: `PermissionSearch::User(user)` yields `cycle:thing:read` and the recursion cut leaves `a` appearing exactly once per path. Delete marker @380.
- Leave markers at `auth/tree/mod.rs:261` and `oxidauth-kernel/src/auth/tree/mod.rs:291` untouched — SRV-3 (dedup), different item.

## Verification

- Before fix, confirm the repro fails-to-complete: `DATABASE_URL=… cargo test -p oxidauth-postgres it_should_recurse_forever_on_a_role_role_grant_cycle` — completes in ≈3 s via the timeout harness (pinned red-behavior proof).
- After fix, the specific regression proofs (scoped, not the full suite):
  - `DATABASE_URL=… cargo test -p oxidauth-postgres role_role_grants` — flipped self-loop/cycle tests pass (23514 + guard error), `it_should_grant_a_child_role_to_a_parent_role` and `it_should_reject_duplicate_pairs_and_unknown_roles` (23505/23503) still pass, proving no over-rejection of legitimate edges or of existing error paths.
  - `DATABASE_URL=… cargo test -p oxidauth-postgres auth::tree` — the terminated-cycle test completes in milliseconds (vs. the old 3 s timeout), and the acyclic fixture tests (`it_should_assemble_user_tree_with_nested_role_grants`, `…role_tree_for_role_search`, diamonds-free fixtures) still produce byte-identical trees, proving the path-local set did not alter DAG output.
  - New permanent test in `insert_role_role_grant/mod.rs`: concurrent-insert race — spawn `A→B` and `B→A` inserts via `tokio::join!`; assert exactly one succeeds and `count_edges == 1` (proves the advisory lock, the non-obvious part of the fix).
  - Migration smoke: against a scratch DB, seed `A→B`, `B→A` raw, run `sqlx migrate run` — must abort with the detection exception naming both edges; after applying the cleanup `DELETE`, re-run must succeed and a raw `INSERT` of a self-loop must now fail with 23514.
- End-to-end smoke on the running server: with an `oxidauth:roles:manage` token, `POST /roles/{A}/roles/{B}` then `POST /roles/{B}/roles/{A}` → second call returns **400** with the cycle message; a login for a user holding `B` still completes and the JWT carries `B`'s entitlements.
- `cargo test -p oxidauth-services role_role_grants` — use-case tests (mock-based, no cycle expectations) stay green unchanged.
