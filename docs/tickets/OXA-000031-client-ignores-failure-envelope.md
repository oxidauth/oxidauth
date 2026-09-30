# OXA-000031 — Raw SDK wrappers return a server failure envelope as `Ok`

**Original ID:** CLI-5 · **Severity:** P2 · **Type:** bug · **Status:** open
**Review tier:** Tier 4 (moderate — multi-file recipes, pinned-test flips) · reviewed 2026-09-30 · DEFERRED (owner decision; claims verified intact at HEAD — 3 raw wrappers (oauth2_redirect, forgot_password, update_password) pass server failure envelopes through as Ok; contract leg-2 pins the bug; Step-1 `check_envelope` recipe sound & small; Step-2 typed ServerError stays bundled here — OXA-000030 closed wontfix, do NOT cite it as dependency; revisit w/ next SDK breaking-window)


## Locations

- **Register drift (confirmed):** the register cites `oxidauth-rs/src/client/users/contract.rs:230` = full path `src/oxidauth/oxidauth-rs/src/client/users/contract.rs`. Line 230 is the `// -- leg 2: BUG(pinned) — the raw wrappers ignore success:false; ...` comment inside the `contract_raw` test harness (`contract.rs:199-249`), i.e. the test that *pins* the defect — not the defect itself.
- **Actual defects** (the three "raw" wrappers, each returning `Ok(result)` straight off the wire):
  - `src/oxidauth/oxidauth-rs/src/client/auth/oauth2/redirect.rs:16-30` — `Client::oauth2_redirect` (post at 25-27, `Ok(result)` at 29)
  - `src/oxidauth/oxidauth-rs/src/client/auth/username_password/forgot_password.rs:13-25` — `Client::username_password_forgot_password` (`Ok(result)` at 24)
  - `src/oxidauth/oxidauth-rs/src/client/auth/username_password/update_password.rs:13-26` — `Client::username_password_update_password` (`Ok(result)` at 24)
- **Checked-path control:** `handle_response` at `src/oxidauth/oxidauth-rs/src/client/mod.rs:650-677` — failure arm at 659-670 turns `!success` into `ClientError { kind: APIResponseError, source: joined error strings }`; 52 in-crate wrapper call sites use it (grep of `handle_response(..)?` outside the definition/tests; the harness doc at `contract.rs:109-111` independently states "52 wrappers" checked vs 3 raw).
- **Transport that ignores status:** `Client::request` at `client/mod.rs:451-496` — never inspects `res.status()`; `.json()` at 485-493 deserializes *any* HTTP status into the caller's type. Errors it *can* produce: `Other("http request failed")` (476-481), `Other("failed to deserialize response")` (488-492).
- **Envelope type:** `src/xlib/http/src/lib.rs:10-27` — `Response<P> { success, payload, errors, warnings, notices, status_code }`; `errors: Option<Vec<serde_json::Value>>` (line 19); `status_code` is `#[serde(skip)]` (25-26), so the HTTP status is unrecoverable after deserialization. Re-exported as `oxidauth_http::Response` via `src/oxidauth/oxidauth-http/src/lib.rs:22`.
- **Server handlers that emit failure envelopes on these exact routes:**
  - `src/oxidauth/oxidauth-api/src/server/api/v1/auth/oauth2/redirect.rs:36` — `Response::bad_request().error(err.into_error())` (HTTP 400, `{"success":false,"errors":[{OxidAuthError}]}`)
  - `src/oxidauth/oxidauth-api/src/server/api/v1/auth/username_password/forgot_password.rs:27` — same 400 failure envelope
  - `src/oxidauth/oxidauth-api/src/server/api/v1/auth/username_password/update_password.rs:24-25` — maps use-case `Err` to `Response::success().payload(UpdatePasswordResponse { success: false })` (see Analysis for why this route is special)
- **Error precedent in the same crate:** `Client::auth` match arm `client/mod.rs:300-316` manually turns `success:false` into `Err(APIResponseError-style ClientError)` with `errors` JSON-serialized into `source`. Related sibling: OXA-000030 (CLI-4, `refresh()`'s `_ => Other("")` arm) — same envelope family, different choke point. **[Review correction 2026-09-30: OXA-000030 closed wontfix — its Step 2 typed-variant proposal will never land; this ticket owns that design decision itself (see Step 2), not a ride on it.]**

## Problem

The three raw wrappers `POST` to an endpoint whose response body is the full `Response<P>` envelope, deserialize it, and hand it back as `Ok` — with no check of `success` and no inspection of the HTTP status. When the server rejects the request, the SDK caller gets:

```rust
// server sends: HTTP 400 {"success": false, "errors": [{"name":"Validation","display":"unknown client",...}]}
let res: Response<Oauth2RedirectRes> = client.oauth2_redirect(params).await?; // <- Ok, `?` passes
// res.success == false, res.payload == None, res.errors == Some([Value(OxidAuthError)])
```

`contract_raw` leg 2 (`contract.rs:230-248`) pins exactly this: mounts `400` + `{"success":"false"... ["boom"]}`, and asserts `.expect("raw wrapper passes success:false through as Ok (pinned bug)")` followed by `assert!(!res.success); assert!(res.payload.is_none()); assert_eq!(res.errors, Some(vec![json!("boom")]))` (244-247). The only way these wrappers return `Err` today is transport/JSON failure from `request()` (`client/mod.rs:476-492`) or the auth state machine — a well-formed server *refusal* always travels the `Ok` channel. Every one of the other 52 wrappers converts the identical envelope into `Err(ClientError(APIResponseError))` via `handle_response`.

## Analysis

- **Mechanism:** `Client::post` → `request` (`client/mod.rs:516-523 → 451-496`). `request` sends, then unconditionally `.json().await` — reqwest does not gate on status, and the failure body *is* valid `Response<P>` JSON (`payload`/`warnings`/`notices` are `Option`, skipped by the server), so deserialization succeeds. The raw wrappers then `Ok(result)` (redirect.rs:29, forgot_password.rs:24, update_password.rs:26). `handle_response` exists precisely to close this gap and was simply never called on these three paths.
- **Why these three are "raw":** unlike resource wrappers, their result type is `Result<Response<P>, BoxedError>` instead of `Result<P, BoxedError>` — the envelope *is* the public return type, so calling `handle_response` (which *unwraps* to `T`) would change the signature. Someone chose to return the envelope wholesale and forgot the `success` check that the envelope's own design implies. The harness doc says it outright: "they never call `handle_response`, so the error leg pins the *pass-through* instead of an error mapping" (`contract.rs:196-198`).
- **HTTP status is double-lost:** `request()` never captures `res.status()` (`client/mod.rs:451-496`), and the envelope's `status_code` field is `#[serde(skip)]` (`xlib/http/src/lib.rs:25-26`). So a fixed envelope check can only key on `success` — 400/401/403/500 all collapse to one shape until status is captured at the `request()` level (same gap OXA-000030 flags for `refresh()`).
- **Per-route exposure against the current server:**
  - `oauth2_redirect`: *live*. Any use-case error (unknown `client_key`, unknown email) → 400 envelope → `Ok` with `payload: None`. The caller's `redirect_url` simply isn't there.
  - `username_password_forgot_password`: *live*. Same 400 envelope → `Ok` with `payload: None` (no `code`).
  - `username_password_update_password`: the handler maps its own use-case errors to `success:true` + inner `UpdatePasswordResponse { success: false }` (`oxidauth-api/.../update_password.rs:24-25`), so the envelope-failure leg is not currently reachable from the handler body; an envelope-shaped `success:false` can still arrive from any layer that answers before/around the handler (auth middleware, proxy) — [INFERENCE: no such middleware returning this envelope shape was located in the repo]. The fix costs nothing on this route and closes the shape.
  - Trap to keep separate: the update-password "wrong code / mismatch" signal rides *inside the payload* (`success: false` on `UpdatePasswordResponse`), not the envelope. The proposed fix deliberately does **not** interpret the payload's own `success` field — that stays caller-side, as today.
- **Related but distinct defects (do not fold in):** the unused-generic turbofish quirk on the two username_password wrappers (`forgot_password.rs:45-47` `BUG(pinned)`, echoed at `update_password.rs:43`); OXA-000030 for `refresh()`; `APIResponseError` carrying no resource/method context (unlike `EmptyPayload(Resource, &'static str)`, `client/mod.rs:603`) — identical Display copy ("error reported when making a request to the API", `client/mod.rs:630-632`) for all 55 endpoints, which a typed-error pass (OXA-000030 Step 2) should fix once for everyone.
- **Mock parity:** `client/mock.rs` and `bin/test_client.rs` do not touch these three wrappers (grep: no hits); no in-repo SDK consumers exist (`oxidauth-cli`, `oxidauth-import-export`, `oxidauth-rs/src/wasm`, no `oxidauth-rs/tests/` — all greps empty). All blast radius is external.

## Impact

- **Who:** external SDK consumers only — these are public API (`pub async fn`, redirect.rs:16, forgot_password.rs:13, update_password.rs:13). The server binary never calls the SDK's copies of these flows (the same-named call at `oxidauth-api/.../oauth2/redirect.rs:16` is the services use-case, not the client). The SDK's intended audience — login/signup/reset UIs — is exactly where silent failure hurts most: an SSO login button that renders nothing (`redirect_url` absent) or a "reset password sent" screen shown after the server said no, with zero diagnostics unless the caller volunteers to check `res.success` and pretty-print `Vec<serde_json::Value>`.
- **Inverted contract:** `Result` says "the `Err` channel is for failures"; here the two most user-visible auth endpoints route business failures through `Ok`. Callers who use `?` and then `.payload.expect(...)` panic on server failure; callers who trust `Ok` show false success. Only callers who manually check `res.success` behave correctly — the type gives no hint that they must.
- **Inconsistency tax:** every other wrapper in the crate raises `ClientError(APIResponseError)` for the same server behavior, so error handling written "the documented way" (see `contract.rs:11-13`, which advertises the error-envelope leg as the crate contract) silently no-ops on these three methods.

## Proposed resolution

**Step 1 — shared envelope check on all three raw wrappers (recommended core fix).** Extract `handle_response`'s failure arm (`client/mod.rs:659-670`) into a sibling:

```rust
// client/mod.rs, next to handle_response
fn check_envelope<T>(resource: Resource, method: &'static str, response: &Response<T>)
    -> Result<(), ClientError>
where T: Serialize + fmt::Debug
{
    if !response.success {
        return Err(ClientError {
            kind: ClientErrorKind::APIResponseError,
            source: response.errors.as_ref().map(|err| { /* same join(", ") as today */ }),
        });
    }
    Ok(())
}
```

refactor `handle_response` to call it (single source of truth for the 52 checked wrappers), and add one line in each raw wrapper before `Ok(result)`:

```rust
check_envelope(Resource::Auth, "oauth2_redirect", &result)?; // Resource::Auth exists, client/mod.rs:560
```

This keeps the public signatures byte-identical (`Result<Response<P>, BoxedError>`) and does **not** add an `EmptyPayload` check — the raw wrappers intentionally keep returning `Ok` with `payload: None` on malformed success (minimal behavior delta; pin that decision with a new leg-3 case, see Verification).

**Step 2 — typed-error upgrade: this ticket now owns it outright.** Prior plan deferred it to OXA-000030 Step 2, but that ticket closed wontfix 2026-09-30, so the host never lands. Decide at implementation time: **own** — add `ClientErrorKind::ServerError { status: Option<u16>, errors: Vec<serde_json::Value> }` + `#[non_exhaustive]` in the same breaking 0.x window, making `check_envelope` the single choke point where all 55 contract wrappers *and* (with a `let status = res.status();` capture in `request()`, `client/mod.rs:471-481`) the status-loss gap are upgraded together — or **drop** Step 2 and ship Step 1 only. Either way, do not cite OXA-000030 as a live dependency.

**Compat analysis (breaking, but compiler-invisible — must be a release-note item, package `oxidauth` v0.9.0 pre-1.0 so a 0.x minor bump is the honest signal):**
- Signatures unchanged → consumer code still compiles; `Ok`-with-failure becomes `Err`. Three caller patterns: (a) checks `res.success` in the `Ok` path — keeps working, their `else` branch goes dead; (b) unwraps `payload` without checking — today panics on failure, tomorrow gets a clean `Err`: strictly better; (c) reads `res.errors` off the `Ok` value to render server messages — **must** migrate to `BoxError::downcast_ref::<ClientError>()` (the harness itself demonstrates the downcast, `contract.rs:178-180`). (c) is the only genuine breakage and is rare: `errors` is `Vec<serde_json::Value>` and useless without custom serialization code.
- If maintainers want a non-breaking path: ship Step 1 as-is (pre-1.0, and `handle_response`'s 52-way convention is the "obvious right" behavior), and *optionally* keep an unchecked escape hatch (`oauth2_redirect_raw` etc.) only if a concrete consumer demands it — none exists in-repo, so default is no shim.

**Pin flips / doc updates:**
- `contract.rs:230-248` (`contract_raw` leg 2, shared by all three wrapper tests): replace `.expect("...pinned bug")` + the three envelope assertions (245-247) with the `contract` runner's error-leg pattern (`contract.rs:178-193` analog): `call(...).await.expect_err("failure envelope must surface as Err")`, `downcast_ref::<ClientError>()`, `matches!(kind, ClientErrorKind::APIResponseError)`, `client_err.source` containing `"boom"`. Keep the one-hit/bearer assertion (248).
- Harness doc comment: `contract.rs:11-13` ("error envelope" leg) becomes true for *all* runners; rewrite `contract.rs:18-20` ("their error leg instead *pins the absence* of error mapping") — it becomes stale on merge.
- Untouched, adjacent pins: `forgot_password.rs:45-47` (turbofish quirk), `update_password.rs:43` comment, `contract.rs:319` (TotpSettings), and all `client/mod.rs` pins (966/1074/1290 — CLI-2/CLI-4/CLI-3 territory, OXA-000028/OXA-000030).

## Verification

- Targeted contract tests (flipped leg 2 runs inside each): `cargo test -p oxidauth oauth2_redirect_route_contract username_password_` — note the cargo package is **`oxidauth`**, not `oxidauth-rs` (`oxidauth-rs/Cargo.toml:2`); the failure fixture (`400` + `{"success":false,"errors":["boom"]}`, `contract.rs:232-240`) already exists in the shared harness — no new wiremock scaffolding needed.
- New leg 3 in `contract_raw`: mount `200` + `{"success":true}` (no payload) and assert `Ok` with `payload.is_none()` — pins the deliberate "check only `success`, never EmptyPayload" boundary.
- Regression on the checked path: `cargo test -p oxidauth handle_response_` (the unit trio at `client/mod.rs:1420-1467`) and the full contract suite `cargo test -p oxidauth client::` must stay green after the `handle_response` refactor.
- Full crate: `cargo test -p oxidauth` (native; wiremock dev-deps are `not(target_arch = "wasm32")`-gated, as OXA-000030 notes).
- Consumer-visible smoke (throwaway, not committed): run `test_client`-style flow against a server returning a canned 400 envelope, confirm `oauth2_redirect` now yields `Err` whose `Display` is the APIResponseError copy and whose `source()` prints `boom`-style server text.
