# OXA-000005 — Unauthenticated forgot_password returns a live TOTP code and wipes all refresh tokens

**Original ID:** SEC-5 · **Severity:** P1 · **Type:** bug · **Status:** Step 1 implemented 2026-09-30 (coder+reviewer approved; Steps 2–3 remain scheduled-remainder, Step 2 unblocked by OXA-000023)
**Review tier:** Tier 4 (moderate — Step 1 hotfix only; Step 2 product fix is Tier 5) · reviewed 2026-09-30 · SCHEDULED — owner approved **Step 1 scope only** (gate route w/ `oxidauth:auth:forgot_password` + blind failure paths; code-in-body stays gated-admin-only until email funds). Step 2 (webhook email, single-use stored code, deferred revocation) stays Tier 5 and needs OXA-000023's count fix. Sequence w/ OXA-000009 + OXA-000042 (same username_password handler family, one pass).


## Locations

All paths relative to `src/oxidauth/` unless noted. Verified against the working tree 2026-09-29.

- `oxidauth-services/src/auth/strategies/username_password/forgot_password.rs:49-79` — `ForgotPasswordUseCase::forgot_password`: fetches the user's TOTP secret (`:54-59`), generates a TOTP code with `.period(600)` (`:63-69`), deletes the user's refresh tokens (`:72-76`), returns `ForgotPasswordResponse { code }` (`:78`). No identity check, no email/webhook dispatch.
- `oxidauth-kernel/src/auth/username_password/forgot_password.rs:10-18` — DTOs: request is `{ user_id: Uuid }`; response is `{ code: String }` and nothing else.
- `oxidauth-api/src/server/api/v1/auth/username_password/forgot_password.rs:15-28` — axum handler. Takes only `State` + `Json(params)`; **no `ExtractJwt` / `ExtractEntitlements`**, unlike gated handlers (e.g. `api/v1/authorities/list_all_authorities.rs:20-22`). Success → `Response::success().payload({code})` (HTTP 200); failure → `Response::bad_request().error(err.into_error())` (HTTP 400, raw error `Display` embedded — `oxidauth-kernel/src/error.rs:32-36`).
- `oxidauth-api/src/server/api/v1/auth/username_password/mod.rs:10` — `POST /api/v1/auth/username_password/forgot_password`, mounted with no middleware.
- `oxidauth-api/src/server/mod.rs:31-38` — the only global layer on the router is `CorsLayer::permissive()`. There is no global auth or rate-limit layer.
- `oxidauth-api/src/server/api/v1/auth/username_password/update_password.rs:15-28` (consumer of the leaked code) and `oxidauth-services/src/auth/strategies/username_password/update_password.rs:112-131` — validates the submitted `code` against the same stored secret with `period(600)`, then overwrites the user's password hash (`:133-160`). Also unauthenticated; all failures return HTTP 200 with `{success:false}` (`:27`).
- `oxidauth-postgres/src/refresh_tokens/delete_refresh_token_by_user_id/delete_refresh_token_by_user_id.sql` — `DELETE FROM refresh_tokens WHERE user_id = $1 RETURNING *` (unconditional, all rows).
- Contrast for the intended design: `oxidauth-services/src/auth/authenticate.rs:152-224` — the login 2FA flow generates the code and **delivers it to the user's email via the authority's webhook** (`WebhookReq { webhook_key, name, email, code }`, `:205-219`); `oxidauth-services/src/totp/validate.rs:112-122` validates it with the authority-configured `totp_ttl`. `forgot_password` skips the delivery step entirely and hands the code to the caller instead.

**Register drift (verified, not assumed):**
- *"full `UserDto` (email, names, status)"* — **does not hold in current code.** The response struct is `{ code }` only (kernel `:15-18`), and no type named `UserDto` exists anywhere under `src/` (grep across the workspace). The TOTP-leak and token-wipe claims are accurate; the `UserDto` claim appears to describe an older revision — git history predates a squashed layout move (`c9312e0`), so it cannot be checked further.
- *"Only a 30s cooldown throttles enumeration"* — **no cooldown/rate limit exists.** No `cooldown`/`throttle`/`rate_limit` code exists in the workspace; `tower-http` is depended on with only the `cors` feature (`oxidauth-api/Cargo.toml:37`); the router layers CORS only (`server/mod.rs:36`). The only "30" in the vicinity is boringauth 0.9.0's *default* TOTP period (`boringauth-0.9.0/src/oath/totp.rs:205`), which this code explicitly overrides to 600 (`forgot_password.rs:65`). The endpoint is fully unthrottled.

## Problem

`POST /api/v1/auth/username_password/forgot_password` is an **unauthenticated, unthrottled TOTP-oracle + session-revoke endpoint**:

1. Anyone who can reach the server may POST `{"user_id": "<uuid>"}`. There is no bearer/JWT check on the route and no server-side gate anywhere between the socket and the handler. (The Rust SDK's wrapper at `oxidauth-rs/src/client/auth/username_password/forgot_password.rs:13-25` *does* attach a bearer — the wiremock contract asserts it, `client/users/contract.rs` leg 1 — so the SDK makes the omission invisible; a raw `curl` succeeds anonymously.)
2. On success the response body contains a **live, current-window TOTP code derived from the user's real enrolled TOTP secret**, valid up to 600 seconds (zero-tolerance `is_valid` semantics; the window-boundary flakiness is acknowledged in the test helper `stay_inside_totp_window`, service tests `:223-233`).
3. As a side effect it **deletes every refresh token the user has** — before any check that the requester is who they claim to be, and even though the "forgot" flow can't even reach a user without a TOTP secret (`select_totp_secret_by_user_id` `fetch_one` errors otherwise → HTTP 400).
4. **No email or webhook is sent.** The register's product question — "gate or deprecate until it emails" — is confirmed: this endpoint implements half of a password-reset flow, and the half it implements is the dangerous half.
5. Chained with its sibling, the code is a **complete account-takeover credential**: `update_password` accepts exactly this code (same secret, same 600 s period) as the sole authorization to set a new password, given `{ username, client_key }` — both of which are configuration-level/low-privilege-read values (client keys are embedded in every SDK client config and readable via permission-gated authority endpoints, not secrets).

## Analysis

**Mechanism.** The use case (`forgot_password.rs:52-78`) treats the response body as if it were the user's mailbox: generate code from stored secret → return it to whoever asked. The service unit test's own comment asserts the safety assumption — *"the auth gate lives in the calling route/permission layer"* (`oxidauth-services/.../forgot_password.rs:237-238`) — but the route has **no gate**, so the assumption is vacuous. This is a broken contract between layers, not a missing check in one place.

**Edge cases found in the current code:**

- **Unknown user / user with no TOTP secret** → `fetch_one` `RowNotFound` → HTTP 400 whose envelope embeds the raw sqlx `Display` (`oxidauth-kernel/src/error.rs:32-36`; see also the `into_error` pins at `:103-106`). The 200-vs-400 split is an existence + TOTP-enrollment oracle for any candidate `user_id`.
- **User with a secret but zero refresh tokens** → the delete query is `fetch_one` over `RETURNING` and **errors when the user has no tokens** (pinned by `oxidauth-postgres/src/refresh_tokens/delete_refresh_token_by_user_id/mod.rs:84` `it_should_error_when_the_user_has_no_tokens`) → HTTP 400 *after* code generation. So the endpoint also reveals whether a user currently holds refresh tokens, and fails outright for them — a functional bug on top of the security one.
- **Duplicate `totp_secrets` rows** (SEC-6) → the arbitrary row returned by `fetch_one` may differ between the `forgot_password` issuance and the `update_password` check, so issued codes can fail verification. Cross-reference SEC-6.
- **Boundary instability:** codes minted in the last seconds of a 600 s bucket expire almost immediately (the `rem_euclid(600) > 595` guard in tests exists precisely because `is_valid` is zero-tolerance). A legitimate user who is slow typing the code gets rejected with no retry path.
- **Replayable within the window:** the code is the raw authenticator TOTP — not single-use, not consumed, not stored. Unlimited `update_password` attempts per window; and since the TOTP secret never rotates on password change, the oracle stays valid across password rotations.
- **Non-atomicity:** secret fetch → code gen → token delete → return; a delete failure surfaces an error while the (untraced, uncounted) side effects have already partially run. The repository cannot even report how many tokens it revoked (BUG(pinned) at `delete_refresh_token_by_user_id/mod.rs:58`).

**Who is affected.** Every deployment exposing `/api/v1`. `user_id`s are `uuid_generate_v4()` (`oxidauth-postgres/src/users/insert_user/insert_user.sql`), so blind brute-forcing IDs is not realistic — but IDs are not secrets: they are the JWT `sub` claim, the payload of `/users` list endpoints, and stored in client-side tokens. Anyone who can observe any user's ID (a co-worker, a leaked token, any endpoint that echoes IDs) can: (a) confirm the account exists and is TOTP-enrolled, (b) revoke all of that user's refresh sessions at will, repeatedly, and (c) if they also know the username + any client key, set a new password within the 600 s window. Ironically the exposed population is exactly the users who enrolled TOTP — and the leak is *their second factor*. Note that refresh-token deletion does not evict live access JWTs (stateless, `jwt_ttl`, `authenticate.rs:233`), so the lockout bites at next refresh — silent, delayed support tickets.

## Impact

- **Confidentiality/integrity:** unauthenticated account takeover (via `update_password` chain) for any account whose `user_id` + `username` + a valid `client_key` are known — no credentials, no rate limit, no alert, no email to the victim.
- **Availability:** anonymous, repeatable revocation of every user's refresh sessions (forced re-login / lockout cycle, no audit trail beyond `tracing`).
- **UX/product:** the "forgot password" flow does not work for its intended user: it only succeeds for users who already have a TOTP secret and refresh tokens, returns an error to everyone else, and delivers nothing to the user's email. It is unusable-as-designed and dangerous-as-built.

## Proposed resolution

**Step 1 — interim hotfix (no product decision required): gate + blind the endpoint.**

1. Add `ExtractJwt` + `ExtractEntitlements` to `api/v1/auth/username_password/forgot_password.rs` and require a dedicated permission (e.g. `oxidauth:auth:forgot_password`, following the `oxidauth:<resource>:<action>` convention enforced by `parse_and_validate` as in `list_all_authorities.rs:27-35`). Seed the permission in the permissions tree/migrations so existing admin roles get it; document that adding the permission string requires a permissions seed/migration. This alone removes the anonymous oracle and unblocks the SDK (which already sends the bearer).
2. Return a **generic success** on any failure path (unknown user, no secret, delete failure): log the real error via `tracing`, but do not forward `err.into_error()` into the 400 body — that stops both oracle variants and stops leaking sqlx internals.
3. Until an email path exists, keep the code in the response *only* for the gated admin caller (support-desk workflow: admin triggers, delivers out-of-band), or skip straight to Step 2.

**Step 2 — product-correct fix: make it actually email.** Reuse the machinery the login flow already has (`authenticate.rs:205-219`): look up the user, build `WebhookReq { webhook_key, name, email, code }` from the authority's `TotpSettings::Enabled` config, POST to the webhook, and return generic `{"sent": true}` with **no code in the body**. Two substantive changes while in there:

- **Stop handing out the raw authenticator TOTP.** Issue a random single-use code stored server-side (own table or hashed column) with expiry + attempt counter, and have `update_password` validate against *that* instead of the user's long-lived secret. This kills the replay window, the "re-ask for the code" behavior, and the coupling to SEC-6's duplicate-secret bug.
- **Move/delay token revocation.** Do not delete refresh tokens on *request*; revoke them when the password is actually *changed* (in `update_password`, after successful verification). That matches user intent ("sign others out because my account was reset") and removes the anonymous DoS primitive. Note revocation accounting needs the delete query to report a count — today it errors on zero rows and `fetch_one`s the first `RETURNING` row (`BUG(pinned)` at `delete_refresh_token_by_user_id/mod.rs:58`, `:84`); switching to `fetch_all`/`EXECUTE`-with-count fixes both together.
    - **Update (2026-09-30, OXA-000023 landed):** this prerequisite is met. `delete_refresh_token_by_user_id` is now `fetch_all`-based with a `Vec<RefreshToken>` response — it reports every deleted row (count via `.len()`, empty `Vec` on zero matches, never `RowNotFound`) and logs `count = deleted.len()`. Both pinned tests (`it_should_delete_only_the_target_users_tokens` marker, `it_should_error_when_the_user_has_no_tokens` → renamed `it_should_return_an_empty_vec_when_the_user_has_no_tokens`) are flipped green and the `BUG(pinned)` marker is deleted; the zero-token forgot-password 400 (`"database row not found"`) is gone — that case is 200 + code now. Step 2's revocation accounting only needs to consume the `Vec`.

**Step 3 — deprecation path (if email is not funded).** If nobody will build the email leg: return HTTP 410 from the route (or remove it), mark `ForgotPasswordParams/Response/ServiceTrait` `#[deprecated]` (precedent: the `#[deprecated]` `response` module in `oxidauth-http/src/lib.rs` for 0.9.0), delete the SDK wrapper + its contract test, and note in the changelog that `update_password` **does not become orphaned**: users with an enrolled authenticator app can still self-generate a valid code, so authenticated password-change-with-2FA keeps working. `update_password`'s current anonymous reachability and its `200 {success:false}` error style are separate findings (adjacent to SEC-4's error-shape pins) — flag, don't silently fix here.

**Compat/migration.** Breaking change either way for anyone calling today; the honest claim is that the only programmatic consumer possible is one exploiting or depending on the leak. SDK: remove/deprecate `username_password_forgot_password` with the same cycle as the kernel DTOs. If Step 1 lands first (gate on), SDK callers keep working *if* their credentials hold the new permission — call that out in release notes since it changes behavior for previously-anonymous calls.

**Pinned-test handling (grep-verified).** No `BUG(pinned)` marker exists on this endpoint's own code (register's "NOT pinned" is confirmed). Related markers and the flip list:

- `oxidauth-rs/src/client/auth/username_password/forgot_password.rs:45-47` `BUG(pinned)` — the un-inferable turbofish generic. **Not part of SEC-5**; leave it (or fix it as its own item). The whole test is deleted only under Step 3.
- ~~flip both~~ **DONE by OXA-000023 (2026-09-30; see Update sub-bullet above)** — `BUG(pinned)` @ `delete_refresh_token_by_user_id/mod.rs:58` and `it_should_error_when_the_user_has_no_tokens` no longer exist; remaining work here is Step 2 only.
- Service test comment `oxidauth-services/.../forgot_password.rs:237-238` ("the auth gate lives in the calling route/permission layer") — non-standard pin, currently **false**; must be rewritten to reference the real route gate added in Step 1.
- Response-shape assertions to flip: service test `returns_a_fresh_totp_code_and_revokes_the_users_refresh_tokens` (`:236-265`, asserts the code equals the current-window TOTP — breaks under Step 2's random-code design; survives Steps 1/3 unchanged) and the wiremock canned payload `{"code":"CODE-123"}` in the oxidauth-rs contract test (must track any response-shape change).
- `update_password` service tests using `stay_inside_totp_window()` (`update_password.rs:450,558,590,632`) — stay valid under Steps 1/3; flip together with Step 2's single-use-code validation.

## Verification

No hurl/e2e coverage exists for this route (`src/oxidauth/hurl/` contains only `public_keys_create.hurl`, `setup_user.hurl`, `tests/`) — coverage is service unit tests + the wiremock client contract only.

- Reproduce the leak (pre-fix, any dev server): `curl -s -X POST $HOST/api/v1/auth/username_password/forgot_password -H 'content-type: application/json' -d '{"user_id":"<known-user>"}'` → expect `200` with `payload.code`, plus the victim's `refresh_tokens` rows gone (`SELECT count(*) FROM refresh_tokens WHERE user_id='<known-user>'`). Post-fix (Step 1): same curl → `401`/unauthorized, zero rows deleted.
- Anonymous chain test (the regression that matters): with a seeded user + client key, anonymous `forgot_password` → `update_password` must no longer complete post-fix.
- `cargo test -p oxidauth-services --lib username_password::forgot_password` — the three service tests (happy path, secret-lookup failure without delete, delete failure).
- `cargo test -p oxidauth-services --lib username_password::update_password` — code-validation contract (Step 2 scope check).
- `cargo test -p oxidauth --lib auth::username_password` (package `oxidauth`, `oxidauth-rs/`) — route/verb/payload contract legs for both wrappers.
- With a test DB (`src/oxidauth/database_test.sh`): `cargo test -p oxidauth-postgres --lib refresh_tokens::delete_refresh_token_by_user_id` — covers the revocation-count/zero-row pinned tests.
- Post-fix manual check that the login 2FA flow is untouched: `cargo test -p oxidauth-services --lib totp::validate` and the `authenticate` tests.
