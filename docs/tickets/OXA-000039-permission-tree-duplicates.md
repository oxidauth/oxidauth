# OXA-000039 — Permission-tree flatten emits one entry per grant path: duplicated entitlements inflate every JWT

**Original ID:** SRV-3 · **Severity:** P2 · **Type:** bug · **Status:** implemented 2026-09-30 (coder+reviewer approved, zero findings; id-keyed HashSet collect() in kernel UserNode/RoleNode flatten, first-occurrence-wins stable order, 3 pins flipped 2→1/5→3/3→2 + diamond test, live-DB tree suite 5/5, cycle-test/OXA-000001 territory untouched, claim-shrink changelog landed)
**Review tier:** Tier 4 (moderate — multi-file recipes, pinned-test flips) · reviewed 2026-09-30 · SCHEDULED (owner approved; scope = id-keyed `HashSet<Uuid>` seen-set collectors in kernel `UserNode`/`RoleNode::permissions()` (first-occurrence-wins, order preserved, dedup-by-id ≡ dedup-by-string via unique_grant_parts); flip 3 pins 2→1 / 5→3 / 3→2 + diamond case; DO NOT touch OXA-000001's cycle test or the tree JSON; changelog line — consumer-visible claim shrink; orthogonal to deferred 000001/000040; 1 kernel file)


## Locations

Register paths are crate-relative; the workspace root nests them under `src/oxidauth/`. All locations verified against the current tree.

| Role | Path | Key lines |
|---|---|---|
| Defect (flatten) | `src/oxidauth/oxidauth-kernel/src/auth/tree/mod.rs` | `UserNode::permissions()` 59–78, `RoleNode::permissions()` 87–106 — nested-first concat (`[permissions, direct_permissions]` 73–76 / 101–104), no seen-set |
| Pinned test (kernel) | same file, `mod tests` | `permissions_are_not_deduplicated_across_nodes` 289–317, `BUG(pinned)` comment @ **291** (register line lands here) — pins a 2× duplicate |
| Pinned test (postgres, user tree) | `src/oxidauth/oxidauth-postgres/src/auth/tree/mod.rs` | `it_should_assemble_user_tree_with_nested_role_grants` 229–277, `BUG(pinned)` @ **261** (register line lands here) — pins `app:thing:read` **3×** |
| Pin-adjacent assertion (role tree) | same file | `it_should_assemble_role_tree_for_role_search` 279–309 — 2× `app:thing:read` (no marker; comment @297–298 defers to the user-tree dedup note) |
| Where the flat list is built | same file | `permissions_as_tree` :38 and :48 call `node.permissions()` to fill `PermissionsResponse.permissions` |
| Grant graph walk | same file | `user_permissions_as_tree` 60–89, `role_permissions_as_tree` 91–121 — one `RoleNode` per *path*, diamonds re-walked |
| JWT mint consumers | `oxidauth-services/src/auth/authenticate.rs` | :226–239 (`PermissionSearch::User` → `.permissions` → `with_entitlements`) |
| | `oxidauth-services/src/auth/register.rs` | :103, :118–121 (AOR inherits this path via `RegisterUseCase`, `authenticate_or_register.rs:54`) |
| | `oxidauth-services/src/refresh_tokens/exchange_refresh_token.rs` | :143, :158–161 |
| | `oxidauth-services/src/totp/validate.rs` | :135, :141–144 |
| Token encoding | `src/oxidauth/oxidauth-kernel/src/jwt/mod.rs` | `with_entitlements` 203–211, `Entitlements::encode` 230–257 (`entitlements.join(" ")` @235), `as_vec` 290–302 |
| Check path | `src/oxidauth/oxidauth-api/src/middleware/can.rs` | `CanService::call` :43–70 → `split(' ')` → `oxidauth_permission::validate` (`oxidauth-permission/src/tokens/mod.rs:57–70`, short-circuit any-match) |
| Downstream extractor | `src/oxidauth/oxidauth-rs/src/axum/extract/mod.rs` | `ExtractEntitlements(pub Vec<String>)` 72–99 — hands the token's raw (duplicated) list to resource-server handlers |
| Schema relevance | `src/oxidauth/oxidauth-postgres/migrations/20221019222410_create_permissions.sql` | :9 `CONSTRAINT unique_grant_parts UNIQUE(realm, resource, action)` — triple ↔ `id` is 1:1 |

**Register drift note:** this line's two cited numbers do *not* point at the defect — they land exactly on the `BUG(pinned)` comment lines (kernel :291, postgres :261). The real defect is the concat-only flatten in `UserNode::permissions()` / `RoleNode::permissions()` (kernel :59–106); the postgres file contains no dedup-able flatten of its own, it just calls the kernel flatten. Also, "3× in the pinned test" is the *postgres* pin; the kernel pin demonstrates 2× (one role path + one direct grant).

## Problem

The permission-tree flatten has no deduplication layer. `UserNode::permissions()` and `RoleNode::permissions()` recursively concatenate `child.roles → flat` then the node's own direct grants, so **a permission reachable via N paths appears N times** in:

1. `PermissionsResponse.permissions` (the flat entitlement array that rides beside the tree, built at `oxidauth-postgres/src/auth/tree/mod.rs:38`/:48), and
2. every JWT issued from it — all four mint paths (`authenticate`, `register`/`authenticate_or_register`, `exchange_refresh_token`, `totp/validate`) take `.permissions` verbatim into `JwtBuilder::with_entitlements`, which does `entitlements.join(" ")` into a single `txt …`/`gz …` claim string (`jwt/mod.rs:235`).

The behavior is pinned in two tests:

- Kernel `permissions_are_not_deduplicated_across_nodes` (:289–317): one permission granted directly to the user **and** via one role → asserts exactly `["oxidauth:shared:read", "oxidauth:shared:read"]`.
- Postgres `it_should_assemble_user_tree_with_nested_role_grants` (:229–277): fixture user → parent → child → grandchild with `app:thing:read` granted at user-direct, parent, **and** child (:152–153 comment "three paths to the same permission") → sorted flatten asserts `read` three times among 5 entries.
- Postgres `it_should_assemble_role_tree_for_role_search` (:279–309) additionally pins 2× `read` within the role subtree (unmarked, same contract).

The postgres pin comment states the intent conflict explicitly: *"audit A3 lists 'dedup' as expected behavior"* — the audit expectation is the contract; the code and pins record the deviation.

## Analysis

**Mechanism.** `RoleNode::permissions()` (mirrored by `UserNode::permissions()`) is:

```rust
let direct_permissions = self.permissions.iter().map(|rp| rp.permission.to_string())…;
let permissions = self.roles.iter().flat_map(|rr| rr.permissions())…;   // recursive
[permissions, direct_permissions].into_iter().flatten().collect()        // pure concat
```

There is no `seen` set at any level, and the recursion restarts a fresh walk per child — the same permission row reached through two role paths, or through a role path plus a direct grant, is stringified once per walk visit. The tree walk itself (`role_permissions_as_tree` :91–121) materializes one `RoleNode` per path, so diamond-shaped grant graphs (`P→X→D`, `P→Y→D`) already duplicate `D`'s entire subtree before the flatten ever runs.

**Check behavior: bloat, not a semantic break.** The server-side gate is `CanService::call` (`api/middleware/can.rs:43–70`): it `split(' ')`s the claim and calls `oxidauth_permission::validate`, which loops `for permission in permissions { match validate_single(…) { Ok(true) => return Ok(true), … } }` (`oxidauth-permission/src/tokens/mod.rs:57–70`) — a short-circuiting any-match. An any-match over a multiset equals an any-match over its deduplicated set, so **allow/deny verdicts are identical with or without duplicates**; duplicates only add redundant parse work (the `Err`-aborts-scan property of `validate` is unaffected — a duplicate is string-identical to its original, so it can neither introduce nor mask a parse error). The same holds for `parse_and_validate_multiple` (:85–98). The real costs are size and the leaked list: `ExtractEntitlements(Vec<String>)` (`oxidauth-rs/src/axum/extract/mod.rs:72–99`) hands resource-server handlers the duplicated vec verbatim, so handler-side `len()`s, UI renderings, and O(n) scans inherit the multiset.

**Token-size mechanism.** Txt (the encoding bootstrap wires, `bootstrap/mod.rs:412`, and what the refresh-claims test pins, `exchange_refresh_token.rs:749–775`) stores the claim as `"txt perm1 perm2 …"`, JSON-escaped and base64url'd into the JWT: each duplicate costs `len(perm)+1` raw bytes ≈ ×4⁄3 in the encoded token (e.g., a duplicate `oxidauth:users:read` ≈ 22 base64url chars per copy, per token, re-minted at every login and every refresh). The pinned fixture wastes exactly 2 extra copies of `app:thing:read` (28 raw bytes) on every token for that user. Real deployments make this unbounded: with diamond fan-out the flat list grows with *path count*, not permission count — a k-layer diamond duplicates a leaf permission 2^k times (the walk cost is OXA-000001's territory, but the *claim* bloat is SRV-3's), and the flat list is otherwise capped only by total grant count, blowing up `Authorization` headers against ~8 KB proxy limits on role-heavy admin accounts. Gz encoding compresses repeats well but still grows the base64 blob and always inflates the decoded `as_vec()` scan.

**Composition with OXA-000001 (SEC-1) — the dedup layer definition.** OXA-000001 proposes a **path-local (ancestor) visited-set** in the postgres tree walk, explicitly "a global set would … change the tree/entitlement output and collide with the separate dedup item SRV-3", and explicitly declines to flip the two markers of this ticket. The layering that composes:

- **Layer 1 — graph walk (OXA-000001):** prunes only edges back into the current path (termination) while keeping the tree a full expansion of grant paths; diamonds keep their repeated `RoleNode`s.
- **Layer 2 — flatten + dedup (this ticket):** collapses the walk's multiset to an entitlement **set** keyed by permission id, first-occurrence order.

The two are order-independent: whichever path first reaches permission P contributes P's single entry under first-occurrence dedup, so OXA-000001's pruning choice cannot change the deduped output, and this ticket must not touch the recursion. Conversely, dedup here is *necessary* even if OXA-000001 later went global: a global visited-set prunes repeated *roles*, not repeated *permissions* reached through distinct roles (user-direct + role grant of the same row is exactly the kernel pin). Deduping at the kernel flatten is also strictly better than deduping inside `Entitlements::encode` or `as_vec`: it fixes `PermissionsResponse.permissions` and the token in one place, and old tokens carrying duplicated claims keep verifying correctly (multiset-safe `validate`), so no token migration is needed.

**Who is affected.** Every JWT mint/refresh path listed above; every resource server using `oxidauth-rs`'s `ExtractEntitlements`; anyone sizing tokens/headers. The tree JSON itself is *not* part of this defect and stays unchanged below (grant DTOs per path are the audit "why" view; only the sibling `permissions` array and the claim change). No HTTP route serves the tree directly (grep: consumers are the four service use cases + tests only).

## Impact

- **Token/header bloat on every token lifecycle** (mint + every refresh), growing with grant-path count, exponentially with diamond depth — worst case breaks header-size limits before any hard cap exists.
- **Confusing consumer-visible output:** duplicated entries in `PermissionsResponse.permissions` and in `ExtractEntitlements`, contradicting audit A3's expected behavior; duplicated lists invite client-side count/`len()` logic bugs.
- **No authz correctness bug** — `validate` semantics are multiset-safe; waste is bounded per-check only by path count.
- Register severity P2 stands: reliability-adjacent cost + contract/audit mismatch, no security or authz break.

## Proposed resolution

**Dedup by permission id at the kernel flatten, first-occurrence wins, stable order.** Confine the change to `oxidauth-kernel/src/auth/tree/mod.rs` `impl UserNode` / `impl RoleNode`; postgres and all service call sites inherit it untouched.

1. Replace the string-concat flatten with one recursive collector per node type, threaded with a shared seen-set:

   ```rust
   // both impls delegate to:
   fn collect(&self, seen: &mut HashSet<Uuid>, out: &mut Vec<String>) {
       for child in &self.roles { child.collect(seen, out); }        // nested first (order preserved)
       for rp in &self.permissions {
           if seen.insert(rp.permission.id) {                        // first occurrence wins
               out.push(rp.permission.to_string());
           }
       }
   }
   pub fn permissions(&self) -> Vec<String> {
       let (mut seen, mut out) = (HashSet::new(), Vec::new());
       self.collect(&mut seen, &mut out);
       out
   }
   ```

   (`std::collections::HashSet`; `Uuid: Hash + Eq`; `use uuid::Uuid` is already imported at :5. Grant DTOs carry `permission: Permission` with `id: Uuid` — see the test helpers at :161–183.)
2. **Why id-key, not string-key:** `permissions.id` is the permission's identity, 16-byte hashing beats per-entry string comparison, and it is the natural key for any future id-based entitlement API. It is *exactly equivalent* to string dedup today because `unique_grant_parts UNIQUE(realm, resource, action)` (`20221019222410_create_permissions.sql:9`) makes id ↔ `realm:resource:action` 1:1 — state this dependency in the code comment so future constraint changes notice it.
3. **Why first-occurrence/stable, not sorted or last-wins:** `user_tree_flattens_nested_roles_before_direct_grants` (:220–228), `role_tree_flattens_nested_grants` (:230–265) and the exact array order in `permissions_response_serde_shape` (:359–366) pin nested-first/direct-last ordering; dedup must only *remove repeats*, never reorder.
4. **Leave the tree (`PermissionsResponse.tree`) untouched** — per-path `RoleNode`s remain the audit view; this ticket changes only the flat list and, transitively, the JWT claim.
5. **Do not touch** `role_permissions_as_tree` recursion, the SQL, `Entitlements::encode/decode/as_vec`, or `oxidauth-permission` — no decode-side dedup is warranted (multiset-safe `validate`; optional belt-and-braces, out of scope).
6. **Pinned-test flips** (only these three assertions change):
   - `oxidauth-kernel/src/auth/tree/mod.rs:289–317` `permissions_are_not_deduplicated_across_nodes` → rename (e.g. `permissions_are_deduplicated_first_occurrence_wins`), drop the `BUG(pinned)` @291, assert `user_node.permissions() == ["oxidauth:shared:read"]`; extend with a diamond (same permission via two sibling roles, none direct) to pin set-size = distinct permissions and first-occurrence order.
   - `oxidauth-postgres/src/auth/tree/mod.rs:261–276` (in `it_should_assemble_user_tree_with_nested_role_grants`) → drop the `BUG(pinned)`, sorted flatten becomes `["app:other:delete", "app:thing:read", "app:thing:write"]`; keep the structural tree assertions (:242–259) unchanged — the tree still shows all three grant paths.
   - `oxidauth-postgres/src/auth/tree/mod.rs:297–308` (in `it_should_assemble_role_tree_for_role_search`) → sorted flatten becomes `["app:thing:read", "app:thing:write"]`; update the comment that defers to the dedup note.
   - **Not flipped:** `it_should_recurse_forever_on_a_role_role_grant_cycle` @ :344 (`BUG(pinned)` @380) belongs to OXA-000001 — do not touch it here.

**Compat:** response array shrinks (bugfix; audit A3 defines the expected shape). Already-issued JWTs keep duplicate claims until expiry/refresh and keep verifying — no rollout ordering, no schema change, no wire-format change. OXA-000001's path-local visited-set is order-independent of this layer; land either first. SRV-4 (`Entitlements::Gz` decode semantics) is adjacent but independent — this ticket neither fixes nor needs it.

## Verification

1. **Kernel unit proof (no DB):** `cargo test -p oxidauth-kernel auth::tree` — the renamed kernel test asserts single-entry dedup + diamond set-size + first-occurrence order; `user_tree_flattens_nested_roles_before_direct_grants`, `role_tree_flattens_nested_grants`, `permissions_response_serde_shape`, `role_tree_serializes_under_the_role_tag` stay green — together they *are* the stable-order regression proof.
2. **Postgres integration:** `DATABASE_URL=… cargo test -p oxidauth-postgres auth::tree` (sqlx spins per-test DBs via `crate::MIGRATOR`) — flipped user-tree (3 sorted entries) and role-tree (2 entries) tests pass; `it_should_return_an_empty_tree_for_a_user_without_grants`, `it_should_error_for_an_unknown_user`, and the cycle test (still pinned-non-terminating) unchanged.
3. **Consumers canary:** `cargo test -p oxidauth-services auth::` — the four mint-path suites use hand-built `MockPermissionTree` responses, so they verify the call sites didn't need changes; `jwt_claims_carry_subject_issuer_authority_ttl_and_tree_entitlements` (`exchange_refresh_token.rs:749`) confirms the claim still mirrors the tree output.
4. **Token-size evidence (throwaway, not committed):** script a `JwtBuilder::with_entitlements(EntitlementsEncoding::Txt, …)` encode of a synthetic 5-path-vs-deduped-3-path entitlement list (postgres fixture strings) and assert the claim string is shorter by `Σ(len(perm)+1)` per removed duplicate; optionally decode via `Entitlements::decode`/`as_vec` to show round-trip parity.
