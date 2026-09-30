# OXA-000060 — seedz fixture collision `ROLE_VIEWER.name == USERS[0].username` ("seedz:viewer" is both a role and a user): no test records it; register lore is stale

**Original ID:** N-10 · **Severity:** n/a · **Type:** note · **Status:** done
**Review tier:** Tier 3 (small precise recipe, multi-file) — ranked #26 of 27 · reviewed 2026-09-30 · IMPLEMENTED (register lore corrected) 2026-09-30 — originally SCHEDULED, rescoped per owner verdict: seedz is (correctly) largely empty — at HEAD the crate is the template stub (`lib.rs` `add()` demo) and the `fixtures.rs`/`main.rs` carrying the collision + S13 tests are UNTRACKED working-tree files (`??` in `git status`), so every code citation in this ticket points at uncommitted WIP — the pin-test/comment work cannot land against HEAD at all, which is the mechanical reason for the rescope. [Review correction: register N-10 lives at `BUGS_AND_NOTES.md:90`, not `:92` — this ticket's three `:92` cites are off by two. Landed correction: at rewrite time the ro…


## Locations

- **Register line:** `BUGS_AND_NOTES.md:92` — "seedz data quirk caught by the validator: `ROLE_VIEWER.name` == `USERS[0].username` — a data collision the old name-based lookup silently tolerated (recorded in seedz tests)."
- **The collision (verified, and it's two pairs, not one):** `src/seedz/src/fixtures.rs` — `ROLE_VIEWER.name = "seedz:viewer"` (`:101-105`, slot 40) vs `USERS[0].username = "seedz:viewer"` (`:128-138`, slot 10, granted `Some(&ROLE_VIEWER)` at `:136`); second pair the register missed: `ROLE_EDITOR.name = "seedz:editor"` (`:106-110`, slot 41) vs `USERS[1].username = "seedz:editor"` (`:140-148`, slot 11, `Some(&ROLE_EDITOR)` at `:146`). `USERS[2]` ("seedz:auditor", `:150-158`) has no role counterpart.
- **Same pattern ships in bootstrap (precedent):** `oxidauth-services/src/bootstrap/mod.rs:307` `ADMIN_ROLE = "oxidauth:admin"` == `:437` `DEFAULT_ADMIN_USERNAME = "oxidauth:admin"` — the project's own provisioning creates the identical role/user name collision.
- **The "validator" as it actually exists:** the S13 fixture-graph test module `src/seedz/src/fixtures.rs:519-714` — `build_run_maps` (`:623-645`, natural-key duplicate asserts `:627-632` permissions, `:638-641` roles), `usernames_and_emails_are_unique` (`:696-702`), `fresh_insert_graph_nodes_are_collision_free` (`:681-693`, PK-level). Provenance write-up: `missing-tests.md:386-396` ("fail the test if a natural-key collision would silently drop a seed row", `:393`). There is **no** code or test named "validator" anywhere (`grep -in 'validator' src/seedz` → 0; `git log -S validator` → 0).
- **Name-keyed lookups today (all single-namespace):** seedz find-then-insert probes `src/seedz/src/fixtures.rs:281-284` (`SELECT id FROM roles WHERE name = $1`) and `:328-331` (`SELECT id FROM users WHERE username = $1`); `GET /roles/by_name/{role}` (`oxidauth-api/src/server/api/v1/roles/mod.rs:27` → `oxidauth-postgres/src/roles/select_role_by_name/select_role_by_name.sql:1-3`); `GET /users/by_username/{username}` (`oxidauth-api/src/server/api/v1/users/mod.rs:31-32` → `oxidauth-postgres/src/users/select_user_by_username_query/select_user_by_username_query.sql:1-3`); bootstrap role find-by-name = list-then-linear-search (`bootstrap/mod.rs:319-325`, with its own `TODO(dewey4iv)` at `:309-310`) and user find-by-username (`:446-450`).
- **Schema:** `users.username` UNIQUE (`oxidauth-postgres/migrations/20221019185709_create_users.sql:5`), `roles.name` UNIQUE (`20221019214702_create_roles.sql:3`) — each namespace dedupes only within its own table.
- **The collision runs green on both endpoints today:** `hurl/variables-local:13` `admin_username=oxidauth:admin`, and one hurl suite run hits `GET /roles/by_name/oxidauth:admin` (`hurl/tests/roles.hurl:19-24`, asserts the ROLE row) and `GET /users/by_username/{{admin_username}}` (`hurl/tests/authenticate.hurl:24,32`, asserts the USER row) against the same DB.
- **Doc touchpoints naming the values:** `changelogs/90000013-seedz-dev-seeding.md:31-36`; `docs/migration-plan/13-seedz-dev-seeding.md:125-131` (inventory) and `:365-375` (re-gate evidence, incl. the hand-insert "collision DB" run).
- **Markers:** `grep -rn 'BUG(pinned)' src/seedz` → **zero hits** (verified). S13 verdict was PASS with none (`missing-tests.md:395`).

## Problem

The register line bundles three claims; only the first is true today.

1. **The data collision: real.** `ROLE_VIEWER.name == USERS[0].username == "seedz:viewer"`, and the register undercounts — the same holds for the editor pair. (The collision is *not* with an "admin" literal; the admin-shaped instance of this pattern lives in bootstrap, `oxidauth:admin` role + user, cited above.)
2. **"caught by the validator … recorded in seedz tests": only half true, and the recorded half is the opposite claim.** The seedz test module (the S13 "validator") *validates per-namespace natural-key uniqueness* — duplicate permission keys, duplicate role names, duplicate usernames/PKs would fail loudly. **No test asserts, mentions, or pins the cross-namespace equality.** Grepping `seedz:viewer` / the equality into `src/seedz` finds only the constants themselves. The tests silently passed *despite* the collision because per-table keying is exactly what the product's reuse logic needs.
3. **"the old name-based lookup silently tolerated": unverifiable in this repo.** At HEAD, `src/seedz` is the `oxidauth-seed` template stub (committed `lib.rs` is the `add(left, right)` demo; `fixtures.rs` and `main.rs` are untracked working-tree files — `git status --porcelain` → `?? src/seedz/src/fixtures.rs`, `?? src/seedz/src/main.rs`). No version-controlled code in this repository ever resolved a single string against both `roles.name` and `users.username`. The "old lookup" is plausibly lore from the parkinglot template lineage (`fixtures.rs:12-13` and `changelogs/90000013:27` cite "parkinglot convention" for the `f0…` UUIDs) — [INFERENCE; no artifact in-tree].

So the actual defect carried by this note is a **stale register entry pointing at a guard that does not exist**, on top of a data collision that current code makes inert (Analysis) but nothing documents as intentional.

## Analysis

**Why the collision cannot bite the product as it stands.** Every mechanism that keys on a *string* is scoped to exactly one namespace, verified exhaustively:

| Namespace | Keyed surfaces | Can see the other side? |
|---|---|---|
| `roles.name` | `roles/by_name` route+SQL; bootstrap list-search `:319-325`; seedz `SELECT id FROM roles` probe | No — query/text is roles-only |
| `users.username` | `users/by_username` route+SQL; bootstrap `find_user_by_username`; seedz `SELECT id FROM users` probe | No |
| login `user_identifier` | `user_authorities.user_identifier` VARCHAR (OXA-000025) — *valued from* usernames by the username_password strategy, never from role names | No |
| permission triple | `realm:resource:action` strings in `can`/entitlements (`demo:dashboard:view`, `oxidauth:**:**`) — a third namespace entirely | No |

Grant plumbing is id-only everywhere: tables `user_role_grants(user_id, role_id)` etc., HTTP `POST /users/{user_id}/roles/{role_id}` (`hurl/tests/user_roles.hurl:46`), seedz threading live ids via `role_ids`/`permission_ids` maps (`fixtures.rs:174-195`). **Permissions cannot be "granted by name" in any dev flow** — the assignment's worry has no in-tree code path. The empirical proof: the admin collision (`oxidauth:admin`) already runs under full load — hurl exercises `by_name/oxidauth:admin` (role) and `by_username/oxidauth:admin` (user) in one green suite, on a seedz-seeded DB (`docs/migration-plan/13:375`, "Succeeded 1 + 17 + 17, Failed 0").

**Seed-time safety under the collision.** seedz's find-then-insert contract (`fixtures.rs:4-13` module docs; `:281-284`, `:328-331`) resolves live ids per table, so a hand/API-created *user* named `seedz:viewer` before the first seed reuses the user row for user-grants, and the *role* probe independently reuses/creates the role row — the re-gate matrix ran exactly this "collision DB" scenario and verified grant targets (`docs/migration-plan/13:369-374`). No cross-wiring is possible: user usernames never enter the `role_ids` map and vice versa (`fixtures.rs:187-191`).

**Where it can still sting: humans and future features, not code.** (a) The bare string `seedz:viewer` is endpoint-ambiguous in the dev loop — the wrong endpoint returns the other entity type or a 404 (confusing, but never a wrong-row *within* a namespace); (b) `psql` probes must qualify table/column; (c) any *future* unified-principal feature (autocomplete, audit "principal" column, import by name) would face genuine ambiguity — the repo's only by-name-identity TODO, `TODO(dewey4iv)` at `bootstrap/mod.rs:380-381` (cross-ref OXA-000022 step 3), targets authority *names*, a third namespace, so it does not widen this one.

**The rename option and its real blast radius.** Strings `seedz:viewer`/`seedz:editor` exist in exactly three files: `fixtures.rs` (constants), `changelogs/90000013:31-36`, `docs/migration-plan/13:125-131` (grep-verified; hurl/clients/helm/bin never reference seedz names, `:293-297` of the plan records the hurl-name collision check too). Code churn is therefore 2–4 literals + 2 doc strings. But the **data** cost is not zero: seed UUIDs derive from `slot`, not name (`fixtures.rs:38-44`), so renaming either side against an existing dev DB makes `find_id` miss on the new name, then the INSERT re-uses the same deterministic `f0…{slot}` PK → `users_pkey`/`roles_pkey` duplicate-key abort. Every seeded dev DB must be reset (or rows hand-deleted) once per rename — a one-time break of the idempotency contract the module docs sell (`fixtures.rs:4-13`). And renaming only seedz while bootstrap keeps the identical `oxidauth:admin` pair is incoherent; renaming bootstrap's pair too would touch the register-flow user (SRV-14 area), the SDK contract test's `"admin"` string (`oxidauth-rs/src/client/roles/find_role_by_name.rs:69-72` — mock-only, but cosmetic drift), and the whole local-dev lore, to fix nothing that breaks.

**Disposition: KEEP the collision as an intentional regression fixture, and make the register's parenthetical true** — today it references a test record that does not exist; the honest resolution is to *write the record*, then retire the note.

## Impact

- **Product behavior today: none.** No query, route, grant, or login path resolves across the two namespaces (exhaustive table above; zero BUG(pinned) hits in `src/seedz`). This is a record/conventions item, P-none; it stays "n/a" severity honestly.
- **Maintainers (the real cost).** A register line that asserts a nonexistent guard is a false search-signpost: a reader greps for the "recorded" test, finds the *per-namespace* uniqueness tests, and either "discovers" the collision as an unknown quirk or (worse) assumes some validator blesses cross-namespace uniqueness and builds on it. An unguarded, undocumented-but-load-bearing-looking data shape invites drive-by renames whose PK hazard (Analysis §rename) lands as a seedz hard-fail on teammates' DBs.
- **Dev UX.** Typing `seedz:viewer` needs the right endpoint; mitigated by a comment + test name (below) and no worse than the `oxidauth:admin` pair devs already juggle.
- **Not affected:** bootstrap flow, hurl suite, SDK contract, `can`/entitlements, invitations — all namespace-scoped (verified above).

## Proposed resolution

**1. Record it (the half the register claims already happened).** In `src/seedz/src/fixtures.rs`:
- Comment at the `ROLES`/`USERS` constants (`:112`, `:128`): role names deliberately equal the username of the user they demo (mirrors bootstrap's `oxidauth:admin` role/user pair, `bootstrap/mod.rs:307`/`:437`); the namespaces are resolved by route/table, never by a shared name; **rename both sides of a pair together** — slot-derived PKs make one-sided renames duplicate-key against existing dev DBs (Analysis).
- New test in the existing `mod tests`, e.g. `role_names_deliberately_match_their_demo_usernames`: `assert_eq!(ROLE_VIEWER.name, USERS[0].username); assert_eq!(ROLE_EDITOR.name, USERS[1].username);` plus `assert_eq!(USERS[0].role.map(|r| r.name), Some(ROLE_VIEWER.name))` — the equality becomes an intentional contract in the repo's established pin style (`permission_key_is_realm_resource_action` `:589-597` already pins exact natural-key strings). This is the regression fixture: any future unified-name code or careless rename trips a named test.
- One line in the module docs (`fixtures.rs:1-13`) stating the same, so API/doc readers meet it before the psql surprise.

**2. Retire N-10.** Replace `BUGS_AND_NOTES.md:92` with (or delete in favor of): "seedz fixture roles `seedz:viewer`/`seedz:editor` deliberately share names with `USERS[0]`/`USERS[1]` usernames (house style of bootstrap's `oxidauth:admin` role+user pair); all lookups are per-namespace so the collision is inert; pinned in `fixtures::tests`." Drop the "caught by the validator / old name-based lookup" lore — the validator (`missing-tests.md:386-396`) validates per-namespace keys only, and no cross-namespace lookup has ever existed in this repo's history (stubs-at-HEAD; [parkinglot-lineage provenance remains an INFERENCE]).

**3. Rejected:**
- *Rename one side of each pair:* ~6 literal/doc edits, a mandatory reset-db cycle on every dev machine (PK-reuse duplicate-key abort, Analysis), asymmetric with the identical bootstrap pair, zero behavior gained.
- *Rename both seedz and bootstrap pairs:* second-order blast radius (hurl asserts at `roles.hurl:19-24`, `variables-local:13`, SDK contract string, registration flow fixtures) to fix a shape the project deliberately ships twice.
- *Add a cross-namespace uniqueness test* (forbid role.name == any username): actively wrong — it would forbid the house pattern, break bootstrap, and invent a constraint no product query depends on.

**Coordination:** none of the files here is touched by OXA-000053 (N-3, which is `oxidauth-postgres` test helpers, not seedz); `fixtures.rs:49` string is OXA-000033's concern, names are not. Register hygiene shared with the §5 notes family (N-1/N-2 → OXA-000051/52, N-3 → OXA-000053): retire line 92 in the same commit as the test.

## Verification

- `cargo test -p seedz` — new pin passes; the existing 12 tests (`fixtures::tests` 9 + `lib::tests` 3, `missing-tests.md:388`) stay green (test-only diff; `cargo check` does not compile `#[cfg(test)]` — use a real run, per T-3).
- **Inertness gate (should already hold, re-run at fix time):** every name-keyed SQL hits one table — `grep -rn 'WHERE name = \$1\|WHERE username = \$1' src/` returns only `select_role_by_name.sql`, `select_user_by_username_query.sql`, and the two seedz probes; `grep -rn "seedz:viewer\|seedz:editor" .` hits only `fixtures.rs`, `changelogs/90000013`, `docs/migration-plan/13`.
- **Live both-namespaces probe** on a seeded stack (`docker compose --profile seed run --rm seedz`): `GET /api/v1/roles/by_name/seedz:viewer` → the ROLE (uuid `f0…28`-band slot 40), `GET /api/v1/users/by_username/seedz:viewer` → the USER (slot 10) — two distinct rows, no cross-talk; the identical probe for `oxidauth:admin` is already pinned green by `hurl/tests/roles.hurl:19` + `hurl/tests/authenticate.hurl:24` (`bin/hurl.sh` green on the collision-seeded DB, `docs/migration-plan/13:373-375`).
- **Register hygiene:** `BUGS_AND_NOTES.md:92` updated/removed; `grep -n 'N-10' BUGS_AND_NOTES.md` reflects the retired wording; the new test name is the single searchable anchor.
- `grep -rn 'BUG(pinned)' src/seedz` still zero (nothing here is broken-behavior-pinned; the new test records intentional data).
