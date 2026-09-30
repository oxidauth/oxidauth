# OXA-000056 — N-6's "can caches decisions 30s" describes no cache that exists; the permission path is stateless and revocation latency is JWT-TTL — retire the note (KEEP, with the real latency invariant folded into OXA-000009)

**Original ID:** N-6 · **Severity:** n/a · **Type:** note · **Status:** done
**Review tier:** Tier 1 (docs-only), ranked item 6/9 · IMPLEMENTED — can.rs comment survived OXA-000069 rebuild (verified 2026-09-30 review pass)


## Locations

All paths relative to `src/oxidauth/`. Verified against the working tree 2026-09-29.

Register line (`BUGS_AND_NOTES.md:88`): *"`can` middleware caches decisions 30s — disabling a user/authority takes up to 30s to take effect (both states pinned in the test; document as intended or add invalidation)."*

The two `can` middleware implementations (there is no cache in either):

- `oxidauth-api/src/middleware/can.rs:52-69` — `CanService::call`: splits `req.permissions()`, calls `validate(&self.permissions, &permissions)` (`:59`), maps `Ok(true)` → next service, `Ok(false)` → `CanError::Unauthorized` (`:66`). Zero I/O, zero state, zero time component — the struct (`:36-39`) holds only `permissions: Vec<Token<'a>>` + the inner service.
- `oxidauth-kernel/src/service.rs:70-88` — the deprecated kernel twin (`#[deprecated(since = "0.5.0", … "removed for 2.0")]` at `:24-27` and `:51-54`), byte-for-byte the same stateless logic.

Where the checked permissions actually come from (the live enforcement path):

- `oxidauth-api/src/middleware/permission_extractor.rs:63-75` — `ExtractEntitlements::from_request_parts`: reads the **`entitlements` claim embedded in the bearer JWT** (`jwt.entitlements…as_vec().unwrap_or_default()`, `:69-72`). No database lookup — nothing to cache and nothing to invalidate.
- `oxidauth-api/src/middleware/permission_extractor.rs:28-47` — `ExtractJwt`: decodes the bearer against `ListAllPublicKeysService::list_all_public_keys` **on every request** (`:36-41`), and `oxidauth-postgres/src/public_keys/select_all_public_keys/mod.rs:18` does `fetch_all(&self.db.read_pool())` per call — key material is read live, uncached.
- `oxidauth-services/src/auth/authenticate.rs:226-239` — the entitlement snapshot is taken **at token mint**: non-TOTP branch computes `permission_tree.call(&PermissionSearch::User(user_id))` (`:226-230`) and stamps `with_expires_in(authority.settings.jwt_ttl).with_entitlements(…)` (`:232-239`); the TOTP branch (`:167-174`) issues a short-lived token (`totp_ttl`) carrying only `TOTP_VALIDATE_PERMISSION`.
- Route handlers gate themselves: 45 files under `oxidauth-api/src/server/api/v1/` call `oxidauth_permission::tokens::parse_and_validate` (`oxidauth-permission/src/tokens/mod.rs:72`) against `ExtractEntitlements` (exemplar `server/api/v1/roles/create_role.rs:17-30`). The `CanLayer`/`ExtractPermissions` middleware itself has **no production wiring at all** — the only `impl ExtractPermissions` in the tree are the two test mocks (`oxidauth-api/src/middleware/can.rs:102-106`, `oxidauth-kernel/src/service.rs:117-121`). The HTTP analogue `GET /can/{permission}` (`server/api/v1/can/mod.rs:17-39`, nested `server/api/v1/mod.rs:28`) is likewise a stateless claim check.
- `oxidauth-services/src/bootstrap/mod.rs:369` — `DEFAULT_JWT_TTL: Duration = Duration::from_secs(60 * 2)` (per-authority override: `authority.settings.jwt_ttl`, `oxidauth-kernel/src/authorities/mod.rs:32`).
- Client-side token store (the only real "cache" in the system): `oxidauth-rs/src/client/mod.rs:425-438` — `check_auth_state` reuses the stored JWT until `now > jwt.exp` (`:435-437`), then refreshes (`:329`); pinned by `authenticate_success_stores_jwt_and_caches_it` (`:818-849`, "caching pin" comment `:843-844`).

What "disabling" touches today — nothing, for either entity:

- `grep` over the full tree for any production branch on either status enum finds only CRUD defaulting/backfill and serde: `oxidauth-services/src/authorities/create_authority.rs:34-36,140`, `oxidauth-services/src/authorities/update_authority.rs:66-68`, `oxidauth-services/src/users/update_user.rs:54-56`, `oxidauth-kernel/src/authorities/mod.rs:66-92` (Display/FromStr), `oxidauth-kernel/src/users/mod.rs:80-91`. **No production code reads `users.status` or `authority.status` to make a decision** — verified independently here, consistent with (and extending) OXA-000009's finding "Where `users.status` *is* read for enforcement: nowhere."
- The auth-path authority lookups are unfiltered: `oxidauth-postgres/src/authorities/select_authority_by_client_key/select_authority_by_client_key.sql` and `…/select_authority_by_strategy/select_authority_by_strategy.sql` contain no `status` predicate (verified by grep; the full-file reads confirm it), and `oxidauth-services/src/auth/authenticate.rs:107-113` branches only on not-found, never on `authority.status`. Same for `oxidauth-services/src/refresh_tokens/exchange_refresh_token.rs` and `oxidauth-services/src/totp/validate.rs` (zero status reads outside test fixtures).

The pins the register points at:

- `oxidauth-kernel/src/service.rs:123-161` and `oxidauth-api/src/middleware/can.rs:108-145` — the twin allow/deny tests (`**:**:**` → `Ok`, `oxidauth:**:write` vs required `oxidauth:users:read` → `CanError::Unauthorized`). "Both states pinned in the test" maps onto these: they pin that the decision is a pure function of (layer permission, request permission) — i.e., **stateless**, the opposite of a cache.
- No test anywhere pins a 30 s window. Every `Duration::from_secs(30)` in the tree is unrelated (`totp_ttl` fixtures, an `NbfOffset::Enabled(30)` fixture `oxidauth-services/src/auth/authenticate.rs:424`, a `jwt_ttl: 30` test fixture `oxidauth-postgres/src/authorities/select_authority_by_id/mod.rs:52,78`, a jwt `leeway` test `oxidauth-kernel/src/jwt/mod.rs:537`).
- Repo history (540 commits, not shallow): `git log --all -S "from_secs(30)"`, `-S "cached_at"` → zero commits; `git log -i --grep cache` → only "add no-cache to build script". A can-path decision cache is not merely absent now — there is no evidence it ever existed in this repository's history.

## Problem

N-6 records that the `can` middleware caches authorization decisions for 30 seconds, so disabling a user or authority takes up to 30 s to take effect, and asks for either a documentation change ("intended") or an invalidation hook.

None of that holds against the current code:

1. **There is no decision cache.** Both `CanService`s validate in-memory claims per request; the checked permission set is a snapshot **inside the JWT** (minted by `authenticate`), and the signature key set is fetched from the database on every verify. `can` performs no lookup that *could* be cached, so there is no TTL, no key granularity, and no invalidation to add.
2. **"Disabling takes up to 30 s to take effect" understates reality — disabling takes effect *never*, for either entity, and not because of any middleware.** `users.status` is enforced nowhere (OXA-000009); `authorities.status` is likewise never read outside CRUD round-trips, so a Disabled authority's `client_key` still authenticates and its tokens still mint normally. The register line implicitly credits `can` with enforcement power it neither has nor needs.
3. **The revocation latency that does exist is the JWT's own lifetime, not a cache.** For bearer auth, a revoked-but-already-minted token is valid until `exp` = `authority.settings.jwt_ttl` (bootstrap default 120 s; the client store reuses it until expiry, `oxidauth-rs/src/client/mod.rs:435-437`). A hypothetical 30 s permission cache would be *faster* than the default TTL — the claim describes the wrong mechanism for the delay it worries about. (An emergency lever exists by accident of the uncached design: `ExtractJwt` reads public keys live, so deleting a key row in `oxidauth-postgres` kills verification of tokens signed with it immediately — global, not per-user, and blunt.)

So this is a **stale register line**, not intended behavior worth defending: the note describes machinery the codebase never had (at least never in recorded history), and the enforcement gap it gestures at belongs entirely to OXA-000009 (user status) and the parallel, currently unticketed fact that **authority status is equally unenforced**.

## Analysis

Mapping the register's two options onto reality:

- **"Add invalidation" — vacuous.** Invalidation presupposes a cache; the only time-bounded state in the decision path is the token's `exp` claim and the client's reuse of it (`check_auth_state`), which is the *design* of stateless JWTs, not a defect. An invalidation hook on status-update use cases (`update_user`, `update_authority`) would have nothing to write into.
- **"Shorten TTL" — an operational knob, not a code change.** `authority.settings.jwt_ttl` is already per-authority configuration, and the honest framing of "max staleness of any revocation" = `jwt_ttl`, whatever gate OXA-000009 eventually adds. Note the asymmetry the OXA-000009 fix must live with: a status gate at authenticate/refresh stops *new* mints instantly, but minted access tokens ride out their TTL; only key deletion (or a jti denylist, which does not exist) shortens that.
- **"Document as intended" — closest, but the thing to document is not a cache.** The invariant worth pinning is: *"permission checks are stateless — the decision is recomputed per request from the JWT `entitlements` claim; revocation latency is therefore bounded by `authority.settings.jwt_ttl` and enforced upstream, where status is (not) checked — see OXA-000009."*

Which lookups are cached — none, in either direction worth guarding:

| Lookup on the auth/authz path | Cached? | Verified at |
|---|---|---|
| Permission decision (`validate`) | No — pure function of claims | `oxidauth-api/src/middleware/can.rs:52-69` |
| Entitlements (`can` input) | No DB lookup — JWT claim | `oxidauth-api/src/middleware/permission_extractor.rs:69-72` |
| Public keys (JWT verify) | No — `fetch_all` per request | `oxidauth-postgres/src/public_keys/select_all_public_keys/mod.rs:18` |
| Authority row (authenticate) | No — queried per request, status unchecked | `oxidauth-services/src/auth/authenticate.rs:107-113` |
| User row | Not loaded at all on auth paths | OXA-000009 ("zero hits" grep, reproduced here) |
| Client JWT store | Yes, until `exp` — pinned, by design | `oxidauth-rs/src/client/mod.rs:425-438,843-844` |

The assignment brief's hint that "authority status IS checked" was checked against the whole tree and is **false**: every `AuthorityStatus` mention outside tests is serialization, CRUD defaulting (`create_authority.rs:34-36`), or backfill (`update_authority.rs:66-68`). No gate exists.

**Disposition: KEEP → retire the note (documentation change; no code change).** There is nothing to fix because there is no cache; "FIX NOW/FIX LATER" would manufacture an abstraction (a cache + its invalidation) that the architecture deliberately avoids. The residual, real concern — revocation latency and disable semantics — is OXA-000009's subject (user side) and a small new register line for the authority side, not `can`'s.

## Impact

- **Behavioral impact today: zero.** The note costs no correctness, security, or performance. Its price is editorial and it is being paid now: a reader triaging the register will budget work to "add invalidation" to a cache that doesn't exist, and may conclude the disable feature has *some* enforcement point with only a 30 s lag — while the truth (nothing enforces either status at all, and `can` never even sees the user row) is strictly worse than what the note advertises.
- **False-confidence risk:** the line implies revocation works after ≤30 s. Someone building an emergency-deprovisioning runbook on that premise gets no revocation at all.
- **Future risk the note half-foreshadows:** if anyone ever moves entitlement checks from JWT claims to live DB lookups, decision caching (and its invalidation) becomes a real design question. That change would surface in the cache-less `ExtractEntitlements` → claim-based design doc first; no code today needs to guard it.

## Proposed resolution

**KEEP (document), and retire N-6 with two edits:**

1. **Replace `BUGS_AND_NOTES.md:88` (N-6)** with the accurate invariant, or strike it and fold the wording into OXA-000009's locations section. Suggested replacement text: *"N-6 (resolved, kept for reference): `can` performs no caching — the permission check is a stateless recomputation per request from the JWT `entitlements` claim; bearer revocation latency is `authority.settings.jwt_ttl` (default 120 s, `oxidauth-services/src/bootstrap/mod.rs:369`) by stateless-JWT design. Neither `users.status` nor `authorities.status` is enforced anywhere at issuance (OXA-000009 + the authority-side equivalent noted here), so 'disabling takes ≤30 s' was wrong in both directions."*
2. **Add the invariant as a comment in `oxidauth-api/src/middleware/can.rs`** (under the existing header block, lines 1-7), so the next person to grep "can cache" finds the answer at the site: e.g. *"Permission checks here are stateless by design: the decision is recomputed per request from the JWT `entitlements` claim (`middleware::permission_extractor::ExtractEntitlements`). There is no decision cache; the staleness bound on any revocation is the authority's `settings.jwt_ttl`, and status enforcement (absent — tracked) belongs upstream in authenticate/refresh, not here."*

No test flips are required by this disposition. If instead a future fix caches anything on this path, the pins to revisit are the twin allow/deny tests (`oxidauth-kernel/src/service.rs:123-161`, `oxidauth-api/src/middleware/can.rs:108-145`) and the client token-store pin (`oxidauth-rs/src/client/mod.rs:818-849`) — none of them pins a cache today.

## Verification

This ticket is documentation-only; it prescribes no code change, so there is nothing to build or run. Facts were established by reading/greping the working tree (2026-09-29):

1. **No cache in either `can`:** read `oxidauth-api/src/middleware/can.rs:52-69` and `oxidauth-kernel/src/service.rs:70-88` in full — bodies are `validate()` only; structs hold no timers or maps. A tree-wide sweep for cache mechanics (`grep -rni "cache\|Instant::now\|UNIX_EPOCH\|from_secs(30)"` across `oxidauth-api/src`, `oxidauth-kernel/src`, `oxidauth-services/src`, `oxidauth-rs/src`, `xlib`) surfaced only unrelated hits (test fixtures, jwt leeway, the client JWT store); sole "cache"-named code is the client token store + its `:818` test.
2. **Stateless claim path:** `oxidauth-api/src/middleware/permission_extractor.rs:28-75` (verify with live key fetch; entitlements from claim); `oxidauth-postgres/src/public_keys/select_all_public_keys/mod.rs:18` (`fetch_all`, no cache layer).
3. **Snapshot at mint:** `oxidauth-services/src/auth/authenticate.rs:226-239` (`permission_tree` → `with_expires_in(jwt_ttl).with_entitlements(...)`).
4. **No status gate:** tree-wide grep for `AuthorityStatus::`/`.status` outside tests returns only CRUD defaulting/backfill and serde; `select_authority_by_client_key.sql` and `select_authority_by_strategy.sql` contain no `status` predicate; `authenticate.rs:107-113` has no status branch.
5. **History check:** repo is non-shallow (540 commits); `git log --all -S "from_secs(30)"` and `-S "cached_at"` return nothing; `git log -i --grep cache` returns only an unrelated build-script commit.
6. **Pins located, none of them cache-based:** `oxidauth-kernel/src/service.rs:123-161`, `oxidauth-api/src/middleware/can.rs:108-145` (allow + deny states), `oxidauth-rs/src/client/mod.rs:818-849` (client token reuse until `exp`).
7. After applying the proposed resolution: `grep -n "N-6" BUGS_AND_NOTES.md` shows the retired wording, and `sed -n '1,15p' oxidauth-api/src/middleware/can.rs` shows the new invariant comment.

## Decision (2026-09-29) — ACCEPTED (KEEP → retire, docs-only), scheduled; no implementation started
- **Review tier: Tier 1 (docs-only), item 6/9** (pulled from original list position 5; review order differed).
- Step 1 option chosen: **replace N-6 in place** at `BUGS_AND_NOTES.md:88` with the ticket's suggested invariant text (stateless per-request check from JWT `entitlements`; revocation latency = `authority.settings.jwt_ttl`, default 120 s; both statuses unenforced → OXA-000009). Not folded into 000009 — the register keeps the "verified false + real invariant" marker.
- Step 2 approved: invariant comment under the `can.rs` header (lines 1–7). **Constraint recorded: OXA-000069 rebuilds `can.rs` (Service removal) — the rebuild MUST carry this comment forward** (the permission check it describes survives the rebuild).
- No test flips; no cache/invalidation code invented (rejected: "add invalidation" is vacuous without a cache).
- Authority-side enforcement gap (`authorities.status` read nowhere) has no sibling ticket: **flagged for the OXA-000009 review** — decide there whether its status gates extend to authorities or a sibling ticket opens. Not in this ticket's scope.
- Future-trigger recorded: if entitlement checks ever move from JWT claims to live DB lookups, decision caching becomes a real design question; revisit the twin allow/deny pins (`service.rs:123-161`, `can.rs:108-145`) + client token-store pin (`oxidauth-rs/src/client/mod.rs:818-849`).
- Acceptance: `grep -n "N-6" BUGS_AND_NOTES.md` shows retired wording; `can.rs` head shows the comment. Doc-only, nothing to build.
