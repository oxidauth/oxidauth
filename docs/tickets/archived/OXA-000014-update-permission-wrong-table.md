# OXA-000014 — `update_permission.sql` targets `authorities`: every permission update fails and the error is swallowed

**Original ID:** DATA-1 · **Severity:** P1 · **Type:** bug · **Status:** done
**Review tier:** Tier 3 (small precise recipe, multi-file) — ranked #25 of 27 · reviewed 2026-09-30 · IMPLEMENTED (as REMOVAL) 2026-09-30 (owner verdict: permissions are not updateable by design — the §4(A)-"not recommended" option wins; Steps 1–3's corrected dead code NOT to be built). Deletion manifest: `oxidauth-postgres/src/permissions/update_permission/` (mod.rs + .sql) + `permissions/mod.rs:9` decl; `oxidauth-repository/src/permissions/update_permission.rs` (trait/params/empty-error) + `permissions/mod.rs:5` decl. **Doc sweep in scope — the delete must not strand references:** strike the DATA-1 register row in `BUGS_AND_NOTES.md`; update `missing-tests.md`, `migration-plan/05` and `/08`, and OXA-000041 `:66`/`:71` (both annotated at review 2026-09-30 to use the new date).


## Locations

All paths relative to `src/oxidauth/` unless noted. Verified against the working tree 2026-09-29.

The defect (two sites, both in the update-permission leaf):

- `oxidauth-postgres/src/permissions/update_permission/update_permission.sql:1-8` — the entire statement is `UPDATE authorities SET realm = $2, resource = $3, action = $4, updated_at = NOW() WHERE id = $1 RETURNING *`. Wrong table; everything else about the statement (bind order, `WHERE id`, `updated_at = NOW()`, `RETURNING *`) is correct and matches the `permissions` schema.
- `oxidauth-postgres/src/permissions/update_permission/mod.rs:12-20` — runtime `sqlx::query_as::<_, PgPermission>(include_str!("./update_permission.sql"))` + `fetch_one(&self.db.write_pool())`, then `:20` `.map_err(|_| UpdatePermissionError {})?` discards the sqlx error into an empty struct.
- `oxidauth-repository/src/permissions/update_permission.rs:21-22` — `#[derive(Debug)] pub struct UpdatePermissionError {}`: no fields, so it is *structurally impossible* to propagate the cause.

The pin (where the register's `:36` actually points):

- `oxidauth-postgres/src/permissions/update_permission/mod.rs:34-65` — test `it_should_always_fail_because_the_sql_targets_the_authorities_table` (`:35`), `BUG(pinned)` comment at `:36-41`, the pinned assertions at `:54-57` (`result.is_err()`) and `:59-64` (the `permissions` row still exists and still reads `realmA`).

Schema ground truth:

- `oxidauth-postgres/migrations/20221019222410_create_permissions.sql:1-10` — `permissions(id UUID PK, realm VARCHAR(256) NOT NULL, resource VARCHAR(256) NOT NULL, action VARCHAR(256) NOT NULL, created_at, updated_at)` plus `CONSTRAINT unique_grant_parts UNIQUE(realm, resource, action)` and three single-column indexes (`:12-14`). `PgPermission` (`oxidauth-postgres/src/permissions/mod.rs:24-32`) mirrors exactly these six columns.
- `oxidauth-postgres/migrations/20221020180349_create_authorities.sql:1-11` — `authorities(id, name, client_key, status, strategy, settings, params, created_at, updated_at)`. **No `realm`, `resource`, or `action` column exists**, so the statement cannot even be parsed by Postgres.
- Sibling permission SQL all correctly targets `permissions`: `delete_permission/delete_permission.sql:1`, `insert_permission/insert_permission.sql:1-2`, `select_all_permissions/select_all_permissions.sql:2`, `select_permission_by_parts/query_permission_by_parts.sql:2`. `update_permission.sql` is the lone outlier.

There is **no caller above the repository** — `UpdatePermission` is implemented-but-orphaned:

- Kernel declares only `create_permission`/`delete_permission`/`find_permission_by_parts`/`list_all_permissions` (`oxidauth-kernel/src/permissions/mod.rs:7-10`); no update request type exists.
- Services mirror that list (`oxidauth-services/src/permissions/mod.rs:1-4`) — no update use case.
- HTTP DTOs mirror it (`oxidauth-http/src/permissions/mod.rs:1-4`) — no update DTO.
- API router mounts only `GET /`, `GET /{permission}`, `POST /{permission}`, `DELETE /{permission}` (`oxidauth-api/src/server/api/v1/permissions/mod.rs:17-20`) — no `PUT`/`PATCH` (contrast the id-keyed update convention on other resources: `users/mod.rs:28`, `roles/mod.rs:25`, `authorities/mod.rs:26`).
- SDK mirrors it (`oxidauth-rs/src/client/permissions/mod.rs:1-4,44-46`) — `PermissionsTrait` bounds four wrappers, no update.
- hurl coverage `hurl/tests/permissions.hurl` exercises GET/POST/DELETE only (requests at `:18,27,43,50,60,67,74`); no update request anywhere in `src/oxidauth/hurl/`.
- Grep for `update_permission`/`UpdatePermission` across the workspace outside `oxidauth-repository` and `oxidauth-postgres`: **zero hits**. The only construction of the params struct in the tree is the pinned test itself (`mod.rs:46-51`).

**Register drift (confirmed, matches the known pattern):** the register cites `oxidauth-postgres/src/permissions/update_permission/:36` — that line is the first line of the `BUG(pinned)` comment inside the test module (`mod.rs:36-41`), not the defect. The actual defects are `update_permission.sql:1` (wrong table) and `mod.rs:20` + `oxidauth-repository/src/permissions/update_permission.rs:22` (error swallowing). The substantive claim itself is exactly right.

## Problem

`UpdatePermission` can never succeed. The prepared statement references columns that do not exist on the table it names, so Postgres rejects it during Parse on **every** call, and the repository replaces the real error with an empty `{}` struct, making the failure undebuggable:

1. `update_permission.sql:1` says `UPDATE authorities`; `:3-5` set `realm`, `resource`, `action`, none of which exist on `authorities` (`create_authorities.sql:1-11`). The bind/where/returning shape is otherwise written for `permissions` — this is a wrong-table copy-paste, not a different intent.
2. sqlx uses the runtime API (`sqlx::query_as(include_str!(...))`, `mod.rs:12`), so the failure is at **statement preparation against the live server**, on every call, before any row is touched. The crate has the sqlx `macros` feature enabled (`oxidauth-postgres/Cargo.toml:23`) but uses **zero** compile-time `query!` macros — so `cargo build` gives this statement no scrutiny at all; nothing in CI short of the DB-backed test suite could ever have caught it.
3. When it fails, `mod.rs:20` maps *any* `sqlx::Error` (undefined column, connection failure, decode failure, timeout — all identical) into `UpdatePermissionError {}` with no `source`. The caller gets an empty debug struct `UpdatePermissionError` and no SQLSTATE, no message, no log line (`mod.rs:12-20` has no `tracing::instrument`, unlike every sibling query which is instrumented, e.g. `insert_permission/mod.rs:14`).
4. Because the failure happens at Parse, it is transactional no-op: the pinned test's second assertion (`mod.rs:59-64`, row still `realmA`) is accurate — nothing is written anywhere, not even to `authorities`.

The register's headline — "Permissions can NEVER be updated" — is true in the strictest sense: not only is the one prepared code path broken, the entire vertical above it (kernel request, service use case, DTO, route, SDK wrapper, hurl test) was never built, so the fixed repository call is dead code the moment it works.

## Analysis

**Mechanism.** A copy-paste from a select/insert-on-permissions template into the authorities query family (git history is a squashed layout move — `c9312e0`, file first appears in `9658d59` "permissions added" — so the introduction commit carries no extra context). The empty error struct then made the failure symptom-free: a caller today can only learn "`UpdatePermissionError`", which is indistinguishable from a legitimate domain error, guaranteeing this would sit unnoticed.

**Why it is a landmine rather than only a dead bug.** The statement's `WHERE id = $1 ... RETURNING *` shape is fine — the *only* thing wrong is the table name. Two hazard notes:

- If anyone ever adds `realm`/`resource`/`action` columns to `authorities` (or creates a same-named view), this statement silently starts mutating authority rows by UUID — cross-table data corruption. Today Postgres' 42703 (`column "realm" of relation "authorities" does not exist`) is the *only* thing preventing that; the code contributes no safety. (Even in that fantasy the result would then fail to decode into `PgPermission`, but relying on two coincidences is not a design.)
- The swallowing map_err is the *only* error-discarding `map_err` in `oxidauth-postgres` — the other two occurrences in the crate (`user_authorities/select_user_authorities_by_authority_id_and_user_identifier/mod.rs:30`, `users/select_users_by_ids_query/mod.rs:34`) both map *with* the error. The house pattern for query errors is `?` into `BoxedError`, and the crate even ships a purpose-built fixture, `test_fixtures.rs:91` `assert_sql_state(err: BoxedError, expected: &str)`, which downcasts a `BoxedError` to a `sqlx::Error` and asserts the SQLSTATE — used by the sibling insert tests (`insert_permission/mod.rs:45,78-104` asserts duplicate-triple → `23505`). The update path opts out of both the pattern and the testing machinery.

**Edge cases the corrected statement must respect:**

- **Unique triple.** `permissions` has `UNIQUE(realm, resource, action)` (`create_permissions.sql:9`). Updating a permission onto an existing triple must surface SQLSTATE `23505` — today's empty error type cannot express it; `BoxedError` + `assert_sql_state` can.
- **Unknown id.** `fetch_one` on zero rows yields `sqlx::Error::RowNotFound` client-side; that too must survive propagation so a service layer can later render 404 rather than a generic failure.
- **Identity semantics.** A permission's identity *is* its triple — `Permission`'s `Display` and the whole HTTP surface are `realm:resource:action` (`oxidauth-kernel/src/permissions/mod.rs:23-27`; routes use `/{permission}`; `RawPermission` parses the string, `mod.rs:51+`). So "update a permission" is an identity change keyed by `id`, and the interesting property is what happens to grants: `role_permission_grants` and `user_permission_grants` reference `permissions(id)` with `ON DELETE CASCADE` (`20221020012706_create_role_permission_grants.sql:9`, `20221020012656_create_user_permission_grants.sql:9`). An **in-place UPDATE keyed by `id` preserves every grant** (they point at the row, not the string); the only path available today — DELETE then re-POST — silently revokes the permission from every role and every user that holds it. That cascade is the user-facing damage that makes the missing update path matter.
- **Validation is upstream-only by design.** The SQL accepts empty strings for realm/resource/action (`VARCHAR NOT NULL` permits `''`); malformed-string rejection lives in `RawPermission::try_from` (`oxidauth-kernel/src/permissions/mod.rs:51+`), which sibling queries invoke before touching the DB (`insert_permission/mod.rs:17`, `delete_permission/mod.rs:16`). The repo trait takes `UpdatePermissionParams { id, realm, resource, action }` (`oxidauth-repository/src/permissions/update_permission.rs:13-19`) with no validation point — any real product path must validate at the service layer like the siblings do, or a permission `::` can be minted straight into the table.
- **Concurrency.** No optimistic locking; two concurrent updates on one id are last-write-wins. Fine for an admin-only mutation; worth a doc note.

**Who is affected.** Today, no live traffic (no route reaches the code). The affected parties are: (a) operators administering permissions — typo-fixing a permission requires delete+recreate, and DELETE silently strips it from all holders via FK cascade, with no way to rename in place; (b) any future feature that consumes `UpdatePermission` and trusts the trait signature — every failure it sees is `UpdatePermissionError` with zero diagnostic content; (c) the test suite, which currently pins the broken behavior as expected (`mod.rs:35-65`).

## Impact

- **Functional:** permission updates are impossible at every layer — the only repo code path always errors, and no service/route/SDK path exists at all. Register rates P1; I would hold P1 while framing the *shipped* half honestly: the P1-magnitude user pain is "renaming a permission silently un-grants it from everyone" (delete+recreate is the only available workaround), and the always-failing call is currently unreachable.
- **Operational/debuggability:** any caller gets an opaque `UpdatePermissionError{}` — no SQLSTATE, no message, no tracing span — for a defect (a wrong table name!) that is trivially diagnosable if propagated. In production this class of error is indistinguishable from a DB outage.
- **Latent data-integrity:** the statement is one schema change away from corrupting `authorities` rows (see Analysis); the safe failure is Postgres' guard, not the code's.
- **Compatibility:** fixing the SQL and error type touches **zero callers** (grep-verified), so no ripple. Adding an update route would be additive. No migration needed; no schema change.

## Proposed resolution

**Step 1 — point the SQL at the right table.** In `oxidauth-postgres/src/permissions/update_permission/update_permission.sql`:

```sql
UPDATE permissions
SET
    realm = $2,
    resource = $3,
    action = $4,
    updated_at = NOW()
WHERE id = $1
RETURNING *
```

Nothing else in the statement changes; `RETURNING *` matches `PgPermission`'s six columns exactly (`permissions/mod.rs:24-32`).

**Step 2 — stop swallowing; match the crate's error idiom.** Replace the empty error struct with the house `BoxedError` pattern used by every sibling (`insert_permission/mod.rs:9-15`, `select_permission_by_parts/mod.rs:10-16`, and `Service<..., Error = BoxedError>` aliases in the repository crate, e.g. `settings/upsert_setting.rs:8`). Concretely:

- `oxidauth-repository/src/permissions/update_permission.rs` — change the trait to return `Result<Permission, BoxedError>` and **delete `UpdatePermissionError`** (`:21-22`). Safe to change outright: zero implementors beyond `PgPermissionRepository` and zero callers (grep-verified).
- `oxidauth-postgres/src/permissions/update_permission/mod.rs:12-22` — keep the same query/`?` body as `insert_permission/mod.rs:19-25`, i.e.

```rust
#[tracing::instrument(name = "update_permission_query", skip(self))]
async fn update_permission(
    &self,
    params: &UpdatePermissionParams,
) -> Result<Permission, BoxedError> {
    let result = sqlx::query_as::<_, PgPermission>(include_str!("./update_permission.sql"))
        .bind(params.id)
        .bind(&params.realm)
        .bind(&params.resource)
        .bind(&params.action)
        .fetch_one(&self.db.write_pool())
        .await?;
    Ok(result.into())
}
```

  Real SQLSTATEs (`23505` unique-triple conflict, `RowNotFound`) now reach callers and `assert_sql_state`. If the team prefers keeping a typed error, the minimum acceptable form is a struct with a `source: BoxedError` field — an empty struct is never acceptable; recommend against since no consumer exists.

**Step 3 — flip the pinned test (grep-verified: `update_permission/mod.rs:36` is the *only* `BUG(pinned)` marker anywhere under `oxidauth-postgres/src/permissions/`).** Replace `it_should_always_fail_because_the_sql_targets_the_authorities_table` (`:35-65`) wholesale — its two assertions (`is_err()` at `:54-57`, untouched-row at `:59-64`) both invert:

- `it_should_update_a_permission_and_return_the_updated_row`: `seed_permission(&pool, "realmA", "resourceA", "actionA")` (fixture at `test_fixtures.rs:176-190`), update all three parts, assert the returned `Permission` carries the new triple, and assert `updated_at >= created_at` moved (or the returned row equals a follow-up select).
- `it_should_reject_an_update_onto_an_existing_triple`: seed two permissions; update the second onto the first's triple; `assert_sql_state(err, "23505")` (`test_fixtures.rs:91`).
- `it_should_fail_with_row_not_found_for_an_unknown_id`: random `Uuid::new_v4()` ⇒ `Err`, downcastable to `sqlx::Error::RowNotFound`.
- Keep one anti-regression leg (cheap insurance against the landmine): after a successful update, `SELECT count(*) FROM authorities` and a spot-check that no authority changed — proving the corrected statement touches only `permissions`.

**Step 4 — product decision: should an update route exist?** Both defensible answers, and the bug fix should not wait for it:

- *(A) Ship Steps 1-3 only.* The trait becomes correct dead code, usable by the next feature. Coherent but leaves the operator pain (typo fix = un-grant everyone) open. If dead code is objectionable, deleting the whole vertical (trait file, mod, SQL, test) is the alternative — not recommended, because the in-place-rename property (grants follow the row) is genuinely load-bearing for the day the route lands.
- *(B) Complete the vertical (recommended follow-up ticket).* Add `oxidauth-kernel/src/permissions/update_permission.rs` (`UpdatePermission { permission: String /* old triple */, new_permission: String }` or parts), a service use case validating both strings via `RawPermission::try_from` before touching the repo (matching `create_permission.rs`/`delete_permission.rs` use-case shape), and route `PUT /api/v1/permissions/{permission}` in `oxidauth-api/src/server/api/v1/permissions/mod.rs` (path = old triple like its siblings at `:18-20`, body = new triple), resolved by parts → id, then updated by id so **id and grants survive the rename**. Guard with `ExtractJwt` + `ExtractEntitlements` + `PERMISSION = "oxidauth:permissions:manage"` exactly like `create_permission.rs:21-31`. Add DTOs in `oxidauth-http/src/permissions/update_permission.rs`, an `UpdatePermissionTrait` wrapper in `oxidauth-rs/src/client/permissions/`, and an hurl leg in `hurl/tests/permissions.hurl`. The DELETE-cascade workaround (silently stripping grants) is the motivating UX: the release note must warn that delete+create remains destructive to grants.

**Compat/migration.** No SQL migration, no schema change. The only contract change is the trait's error type — zero external implementors/callers, so no downstream break. Test-suite contract change: the pinned test asserting failure must be deleted/flipped in the same commit (Steps 1-3 are one atomic change; the old test cannot coexist with the fix). If Step B lands later, note that a permission renamed under an id keeps every grant — operators who previously relied on delete+recreate to revoke should double-check what `PUT` does in their tooling.

## Verification

No route exists today, so the repository test *is* the end-to-end surface until Step B; there is no existing hurl/SDK coverage to update (confirmed: `hurl/tests/permissions.hurl` has only GET/POST/DELETE, SDK has no update wrapper).

- Before fixing, reproduce: `cargo test -p oxidauth-postgres --lib permissions::update_permission` (DB required — `src/oxidauth/database_test.sh` provides it; tests run against `crate::MIGRATOR`, `oxidauth-postgres/src/lib.rs:45`) — the pinned test passes (failure is asserted). After Steps 1-3 it must be replaced by the Step-3 tests and those must pass: update returns the new triple, `23505` on collision, `RowNotFound` on unknown id, `authorities` untouched.
- Manual repro of the swallowing, to show the fix changes the diagnosis path: on current code, a small scratch binary calling `PgPermissionRepository::update_permission` prints `UpdatePermissionError`; post-fix, the same call path surfaces `error received response ... 42703`-free success or the real SQLSTATE for a collision.
- `cargo build -p oxidauth-postgres` proves nothing here (runtime `include_str!` SQL, no `query!` macros — see Locations); the DB-backed suite is the only compile-plus-prepare proof. Follow-up hardening worth considering once CI has Postgres: gate the merge on `cargo test -p oxidauth-postgres` so any future wrong-table statement fails the suite rather than the pinned-test ratchet.
- If Step B lands: new hurl leg in `hurl/tests/permissions.hurl` (POST a permission, `PUT` it to a new triple, `GET` the new triple → 200, `GET` the old triple → 404, re-`PUT` onto a duplicate triple → the domain error), plus a grants-preservation assertion in the service test (seed a role grant via `role_permission_grants`, rename, assert the grant row still resolves and `can` still authorizes).
