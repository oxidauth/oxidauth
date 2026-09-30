# OXA-000059 — api thin CRUD handlers deliberately untested; hurl e2e is the only coverage layer

**Original ID:** N-9 · **Severity:** n/a · **Type:** note · **Status:** RETIRED 2026-09-30 (priority slice implemented — coder+reviewer approved; resolution item 2 landed in full: 10 modules, 49 handler tests on the callback.rs pattern incl. gate/self-scope/quirk pins + both BUG(pinned) pre-registered for OXA-000042 Step 3, api suite 27→76, zero product-code drift; review minors closed same-day. Item 4 resolved by owner action: `missing-tests.md` removed from the repo 2026-09-30, so the stale D3 row died with the ledger — docs/TESTING.md now carries the convention and re-measured gate numbers; N-9 retires per its own condition)
**Review tier:** Tier 4 (moderate — multi-file recipes, pinned-test flips) · reviewed 2026-09-30 · SCHEDULED with **CHANGED scope** (owner: the trait-for-everything refactor makes services mockable through the provider — the blanket "handlers untested by design" policy is **superseded**; handler tests are now the expectation, mechanism = hand-written `XServiceTrait` mocks registered in `Provider` under the `XService` alias key, precedent already in-tree at `callback.rs:149-433`. See "Resolution revision" below — the exception-clause approach is retired.)

## Resolution revision (owner decision 2026-09-30) — supersedes "Proposed resolution" clauses above

The proposed codified-exception rule is **NOT** adopted. New policy, now expected convention per `docs/TESTING.md` rewrite + D3 row:

1. **Every handler gets a unit test module** using the callback.rs pattern: mock the service trait(s) it fetches, register in `Provider` under the `XService = Arc<dyn XServiceTrait>` alias, drive via `tower::ServiceExt::oneshot` on the real router/route. Minimum per handler: success arm, failure arm, status/body shape, params pass-through; plus every branch the handler actually contains (gates, `jwt.sub` handling, self-scope challenge vectors, both-arms-200 quirk assertions).
2. **Priority order (this ticket's delivery slice — the census's exposed set):** `update_password` (both-arms-200 + gate-less — pairs with deferred OXA-000042's Step 3: whoever lands first writes the handler test, second rebases), `totp/validate` (missing-sub 401), `forgot_password` (un-gated canonical pair), `find_user_by_id` + `find_user_by_username` (self-scope challenge), `oauth2/redirect`, `invitations` ×4 (create/find/accept/delete — covers deferred OXA-000002's zero-e2e blind spot directly). 11 modules.
3. **Remaining handlers** (pure template, hurl-covered): tests expected as touched — any PR opening a handler file adds its test module; no big-bang sweep required.
4. **D3 rewrite records the new convention** (61 handlers, mechanism, callback.rs as the reference pattern, the 8-exception/zero-coverage inventory as the priority list), corrects "~80" → 61. Retire N-9 when the 11 modules + D3 rewrite land.
5. hurl legs for invitations (`hurl/tests/invitations.hurl`) stay recommended-but-optional under the new policy — unit tests now carry the coverage obligation.


## Locations

All paths relative to `src/oxidauth/` unless noted. Verified against the working tree 2026-09-29.

- Register line: `BUGS_AND_NOTES.md:91` (§5 "Notes (not bugs — record for later)", `:81`): "api thin CRUD handlers are intentionally NOT unit-tested (S8/D3): each delegates straight to a service; coverage lives in the hurl e2e suite (17 files)."
- The S8/D3 decision record: `missing-tests.md:259` (row **D3**, `missing-tests.md:367` "S8 api — round 1 — verdict: PASS", `:384` decision recap; companion rows D1 `:257`, D2 `:258`). Date on the decision: 2026-09-29.
- Handlers: `oxidauth-api/src/server/api/v1/**` — **60 handler modules**, each exporting one `pub async fn handle` (counted: `find … ! -name mod.rs -name '*.rs'` → 60; `async fn handle` occurrences → 60), plus the `/can/{permission}` endpoint (`async fn can`, not named `handle`) in `oxidauth-api/src/server/api/v1/can/mod.rs:17-40` → **61 endpoint handlers** total.
- The single unit-tested handler: `oxidauth-api/src/server/api/v1/auth/oauth2/callback.rs:149-433` (`#[cfg(test)] mod tests`, 9 tests). Handler-side coverage also lives in the middleware tests: `oxidauth-api/src/middleware/permission_extractor.rs` (12 tests, incl. the authorities-list permission-gate envelope pinned end-to-end on the real router — D1) and `oxidauth-api/src/middleware/can.rs` (2 tests).
- hurl suite: `hurl/tests/*.hurl` — **17 files**, exactly as the register says (`authenticate`, `authorities`, `can`, `exchange`, `healthcheck`, `livecheck`, `permissions`, `public_keys`, `register`, `role_permissions`, `role_roles`, `roles`, `settings`, `user_authorities`, `user_permissions`, `user_roles`, `users`). On disk there are 19 `.hurl` files: the two extras are `hurl/public_keys_create.hurl` (a runner setup step, invoked separately by `hurl.sh:70`) and `hurl/setup_user.hurl` (orphaned standalone — its own header says "hurl.sh runs only public_keys_create.hurl + tests/*.hurl"; already recorded in `docs/migration-plan/11-scripts-and-tests.md:279` NIT 5, kept per Main N2).
- Router for the coverage comparison: `oxidauth-api/src/server/api/v1/mod.rs:18-32` (12 nested resource routers).
- `BUG(pinned)` census near this area: exactly one marker in the api crate — `oxidauth-api/src/server/api/v1/auth/oauth2/callback.rs:322` (the 200-on-error pin owned by OXA-000037). No pinned markers guard any untested handler.

## Problem

N-9 records a test-architecture decision: `oxidauth-api` handler modules get **no unit tests**; the ~60 thin CRUD handlers are considered "arg-extraction + one `fetch_unchecked` service call + response-map", with real branching pushed into services (tested) and extractors (tested), and end-to-end behavior covered by the 17-file hurl suite. The S8 review formally ACCEPTED this (D3, `missing-tests.md:259`).

As a register item it needs a disposition: is the claim true today, did the acceptance enumerate the exceptions honestly, and is "no handler unit tests, hurl covers it" safe to keep as written? The register statement is **true as to fact but the accepted exception inventory is incomplete**, and the policy has already cost something (OXA-000042's update_password handler) — details below.

## Analysis

**The claim verifies, with two number fixes:**

1. *"each delegates straight to a service"* — true for essentially all CRUD modules. The canonical template (`oxidauth-api/src/server/api/v1/users/update_user.rs:28-73` is representative): `parse_and_validate(PERMISSION, …)` three-way gate → `provider.fetch_unchecked::<XService>()` → build params from `Path`/`Json` → one service call → `Response::success().payload(...)` / `Response::bad_request().error(err.into_error())`. The gate is branching, but it is mechanically identical in every file and its two rejection arms (bodyless 401 from the extractors, `{"success":false}` 401 envelope from `Response::unauthorized()`) are pinned in `middleware/permission_extractor.rs` (D1's correction: both extractors reject with a bare status; handlers mint the envelope — so gate behavior genuinely is extractor-covered).
2. *"17 files"* — correct for the suite (`hurl/tests/*.hurl` globbed by `hurl.sh:73-74`, run twice per invocation so pass 2 only goes green if pass 1 cleaned up). `missing-tests.md:259`'s "~80 thin handlers" is not: the router exposes 61 handler functions, not ~80. [INFERENCE] The audit likely counted http request/response modules or pre-migration handlers; the register should carry the true number.

**Deviation inventory — handlers that are NOT thin delegation (my census of all 61):**

| Handler | Extra logic | Test status |
|---|---|---|
| `auth/oauth2/callback.rs` (handler `:30-104` + helper `gen_redirect_url :106-146`) | query/path extraction, 3 `ERROR_RESPONSE.into_response()` early returns, redirect-URL assembly | **unit-tested** (9 tests); the 200-on-error defect is itself pinned `:322-327` (OXA-000037) |
| `auth/username_password/update_password.rs:23-26` | no permission gate; **both** arms answer HTTP 200; `Err(_) => {success:false}`, `Ok(_) =>` fabricates `{success:true}` | **zero tests** — no unit test, no hurl request (verified: no `update_password` in `hurl/tests/`; recorded in OXA-000042:26), client wiremock contract insensitive to status (OXA-000042:26) |
| `auth/username_password/forgot_password.rs:19-28` | no permission gate; response map itself is canonical | zero tests; no hurl request (OXA-000005:80 records the same gap) |
| `totp/validate.rs:40-43` | no-permission-gate route; **`jwt.sub` required** — absent `sub` → 401 branch outside the template | zero tests; `POST /api/v1/totp/validate` absent from hurl |
| `users/find_user_by_id.rs:25-29`, `users/find_user_by_username.rs:27` | self-scoped challenge leg: `oxidauth:users.<jwt.sub>:read` added next to `oxidauth:users:manage` | zero tests; no hurl request grants a self-scoped permission (`oxidauth:users.` — with the `.uuid` suffix form — never appears in `hurl/`; the only grant leg, `hurl/tests/can.hurl:92`, grants a raw `hrc<stamp>:thing:read`) |
| `can/mod.rs` (`can` endpoint) | real logic with **no service at all**: `parse` + `validate` on the path permission | adequately covered by hurl: `hurl/tests/can.hurl` pins payload true (`:20-27`), payload false + warning (`:30-36`), 400 "invalid permission" (`:39-46`), and a zero-entitlement fresh registrant (`:48+`) |

`redirect.rs:10-38`, `exchange.rs`, `register.rs`, `settings`, `meta`, and the other 50 modules are the pure template. (`create_user.rs`, D3's named candidate, indeed has no branching beyond its service call — D3 was right about it.)

**What hurl does cover, including failures — the suite is not happy-path-only:** no-token 401 (`users.hurl:19-21`), authenticate 401/400/400/422 legs (`authenticate.hurl:33,44,59,73`), duplicate registration → 400 with `$.errors[0].debug contains "duplicate key"` (`register.hurl:56-69`), missing-entity GETs (all-zero uuid ids, `hurl-nobody-` username, `hurl_test_missing_` setting), and full CRUD cleanup on every resource, run twice under a per-invocation `stamp` so residue trips unique constraints. The register.hurl duplicate-key assert is exactly why OXA-000050's fix (typed duplicate errors instead of raw 23505 text) will have to flip a **hurl** assertion — end-to-end pins for thin handlers are real load the suite already carries.

**What hurl structurally cannot cover:**
- **Infra-failure branches.** Business failures reachable on a healthy DB are pinned (see above); an actual repository failure (pool exhaustion, connection death, decode error) is not producible from an HTTP client. That is precisely the channel OXA-000042 abuses: on update_password, a DB outage and a business refusal are indistinguishable from outside, and hurl *cannot* catch a handler that 200s both.
- **Concurrency.** Duplicate-register *races* (both inserts passing or a partial `users`/`user_authorities` write — `oxidauth-services/src/auth/register.rs:90-98` has no transaction) are invisible to a serial hurl pass; the 23505 pin covers only the serialized case.
- **Privilege shapes the suite never logs in as.** Every non-admin identity in the suite is one freshly-registered user; can.hurl grants it a permission, but no identity is ever granted the `oxidauth:users.<own-id>:read` self-scope — the `find_user_by_id/username` self-access leg is untested at both tiers.
- **Provider wiring panics.** Every handler calls `provider.fetch_unchecked::<T>()` — an unregistered service is a panic no e2e run hits unless the endpoint is hit at all, and endpoints outside the suite (below) are never hit.

**Zero-coverage routes (neither unit tests nor a single hurl request):** `/api/v1/invitations` ×4 handlers, `/api/v1/auth/username_password/forgot_password`, `/api/v1/auth/username_password/update_password`, `/api/v1/auth/oauth2/redirect`, `/api/v1/totp/validate` — **8 of 61 handlers are not exercised end-to-end at all** (callback is excluded: unit-tested). These four resource gaps map 1:1 onto tickets that were found *without* test help: OXA-000005 (forgot_password TOTP leak), OXA-000042 (update_password error swallowing), OXA-000043 (redirect params missing).

**Did the policy already cost something — yes, once, exactly at the gap.** OXA-000042:18 documents the handler fabricating `success: true` on HTTP 200 while dropping the service's flag, and OXA-000042:26 that no hurl test touches the route and the client contract test cannot see server status. (The assignment-line paraphrase "No handler tests exist… handler fabricates success" is *not verbatim* in OXA-000042; the two findings above are what the ticket actually says.) Had a 30-line handler test with a mocked `UpdatePasswordService` existed — the callback.rs pattern — the 200/`success:true` inversion would have surfaced at write time. Contrast callback.rs: the one handler that *does* have tests also shipped the sibling 200-on-error bug (OXA-000037), but there it was pinned, ticketed, and its text owned; on update_password the same family of defect sat invisible. Tests don't prevent bad mapping; the absence of tests prevents *noticing*.

**The accepted exception list is already stale.** D3 (`missing-tests.md:259`) claims: "Exceptions with genuine branching are already unit-covered: oauth2 `callback` …, `authorities/list_all` permission-gate envelope …, `can` middleware …" and "No further handler unit tests planned." My census finds three additional handlers with genuine branching never listed (update_password, totp/validate, find_user_by_id/username) and 8 routes with no e2e pin at all — on the **same day** the decision was recorded. An acceptance whose exception inventory is wrong within 24h argues the *rule* needs codifying, not just re-affirming. The friction D3 was managing is real: `oxidauth-api` has no mockall dependency; the callback tests hand-roll `MockAuthenticateOrRegister` implementing the kernel service trait (~40 lines of mock + router harness, `callback.rs:174-272`), so a test per deviating handler is ~60–80 lines — affordable for five files, absurd for sixty.

**Verdict: KEEP the policy, but retire N-9 as a bare note only once it is rewritten with a written exception rule and the gap-closure list below.** "Thin handlers, no unit tests, hurl covers it" is the right boring architecture (a 27-line handler with a P1-class bug proves neither the LOC threshold nor "all exceptions covered"; branching/deviation is the right trigger).

## Impact

- **Present risk:** the 8 zero-coverage handlers are the codebase's blind spot — three already host open P1/P2 defects (OXA-000005, OXA-000042, OXA-000043); invitations host the accept-expired bug (SEC-2, OXA-000002) and have never seen a single request of any kind in CI-adjacent tooling. Infra-error and self-scope branches are untestable in e2e by construction, so nothing closes them except handler tests.
- **Drift risk:** with no written rule, every future handler addition silently inherits "no tests", and the next `update_password`-shaped deviation (new status mapping, no gate, `jwt.sub` logic) will again be discovered by audit, not CI.
- **Cost of the KEEP-plus-rule path is small:** ~4 handler test modules + 2–3 hurl files; no framework changes (harness precedent already in-tree at `callback.rs:174-272`).

## Proposed resolution

**Disposition: KEEP, amended — codified exception rule + targeted gap closure. Do not mandate handler tests wholesale.**

1. **Write the exception rule into the register** (rewrite `BUGS_AND_NOTES.md:91`, `missing-tests.md` D3 row; then retire N-9). A handler module MUST have a handler-level unit test (callback.rs pattern: hand-rolled service mock + `Router` oneshot) if **any** of:
   - **(a)** it branches on a value the template doesn't supply — `jwt.sub` comparisons/self-scope challenges, URL/redirect assembly, any non-helper `if`/`match` beyond the two canonical `match`es;
   - **(b)** either response arm deviates from the canonical pair `Response::success().payload(...)` / `Response::bad_request().error(err.into_error())` — including `Response::success()` on the error arm (the OXA-000037/000042 family — this clause alone would have caught both);
   - **(c)** it defines a non-`handle` function (helper) that participates in the response;
   - **(d)** it omits the permission gate on a route the sibling resources gate (`invitations` style check) — gate-free auth routes must at least pin their Err arm status;
   - **(e)** its route has **zero** end-to-end pin (no hurl request) — then either a hurl file or a handler test, prefer hurl for pure-template handlers, handler tests for ones tripping (a)–(d).
   Line count is deliberately **not** the trigger: `update_password.rs` is 27 lines with the worst bug in the crate; `callback.rs` is 432 and behaved.
2. **Close the enumerated gaps** (each already has an owner ticket; this rule rides along):
   - `update_password.rs` — OXA-000042 Step 3 rewrites both arms; require a handler test module (outage→5xx, business refusal→4xx, happy→200 `{success:true}`) in that ticket's diff. Catches clause (b).
   - `totp/validate.rs` — handler test pinning the missing-`sub` 401 and the canonical arms (clause (a)).
   - `find_user_by_id.rs` / `find_user_by_username.rs` — one handler test each for the self-scope challenge construction; or a hurl leg granting a registrant `oxidauth:users.<own-id>:read` (clause (a)/(e) — pick one, hurl preferred since the route already has a hurl file).
   - **invitations** — add `hurl/tests/invitations.hurl` (create → find → accept → delete round-trip, one file, four handlers covered, cleanup under the stamp convention — clause (e), pure-template so hurl is the right tier).
   - **forgot_password / update_password / oauth2 redirect** — add a `hurl/tests/username_password.hurl` + redirect leg: happy path + the *current* Err-arm status, so future status changes trip something (clause (e)); content bugs stay in OXA-000005/000042/000043.
3. **Fix the numbers** wherever the decision is restated: 61 handler functions (60 `handle` modules + `can`), 1 unit-tested handler, 17 suite files + 2 helpers, D3's "~80" → 61.
4. **Alternative considered — targeted exceptions only, no written rule** (i.e., KEEP N-9 verbatim + list the 5 exceptions): rejected because the last exception list compiled by review was wrong on its own acceptance date (D3 vs. my census), and exceptions with no membership criterion don't survive the next handler added.

## Verification

Reproduce the census (read-only; no builds/tests run for this ticket):

- `find src/oxidauth/oxidauth-api/src/server/api/v1 -name '*.rs' ! -name 'mod.rs' | wc -l` → 60; `grep -rc 'async fn handle' src/oxidauth/oxidauth-api/src/server/api` sums to 60; the `/can` endpoint `async fn can` lives in `can/mod.rs` → 61 total.
- `grep -rl 'cfg(test)' src/oxidauth/oxidauth-api/src` → exactly `middleware/permission_extractor.rs`, `middleware/can.rs`, `server/api/v1/auth/oauth2/callback.rs` (9 tests: `grep -c '#\[\(tokio::\)\?test\]'`).
- `find src/oxidauth/hurl -name '*.hurl' | wc -l` → 19; `ls src/oxidauth/hurl/tests/*.hurl | wc -l` → 17 (the runner globs only `hurl/tests/*.hurl` + `public_keys_create.hurl`, `hurl.sh:70-74`; `setup_user.hurl` self-documents as orphaned).
- Zero-coverage set: extract verb+URL lines from `hurl/tests/*.hurl`, diff against `route(` registrations under `server/api/v1` — expect exactly the 8 handlers listed (invitations ×4, forgot_password, update_password, redirect, totp/validate).
- Error-branch pins hurl already carries: `register.hurl:56-69` (duplicate → 400 + `duplicate key`), `authenticate.hurl:33-73` (401/400/400/422), `users.hurl:19-21` (bodyless 401), `can.hurl:20-46` (true/false/400).

After the resolution lands: `grep -rl 'cfg(test)' src/oxidauth/oxidauth-api/src/server/api/v1` grows by the four exception modules; `ls hurl/tests/` grows by `invitations.hurl` (+ username_password legs); `BUGS_AND_NOTES.md` §5 no longer contains the N-9 line (retired by its rewrite into the rule text), and OXA-000042's Step-3 diff includes the handler test module. A cheap future CI guard (out of scope here): a script that enumerates `route(` registrations and fails any handler file that neither matches rule clauses (a)–(e)-clear nor appears in a hurl request.
