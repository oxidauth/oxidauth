# OXA-000030 — refresh() collapses every server-reported failure into an empty error string

**Original ID:** CLI-4 · **Severity:** P2 · **Type:** bug · **Status:** closed (wontfix — owner decision, 2026-09-30)
**Review tier:** Tier 3 (small precise recipe, multi-file) — ranked #23 of 27 · reviewed 2026-09-30 · REJECTED — wontfix by owner; claims verified, Step 1 plan + corrections kept for a revisit


## Locations

- **Actual defect:** `src/oxidauth/oxidauth-rs/src/client/mod.rs:419` — catch-all arm of the response `match` in `Client::refresh()`: `_ => return Err(ClientError::new(ClientErrorKind::Other(""), None))`. This is the only `ClientErrorKind::Other("")` construction site in the workspace.
- **Register drift:** the register cites `oxidauth-rs/src/client/mod.rs:1074`, which is the `BUG(pinned)` comment inside the wiremock test `refresh_failure_envelope_collapses_to_an_empty_error` (test spans `client/mod.rs:1053-1083`), not the defect. Confirmed drift pattern.
- **Display wart:** `ClientErrorKind::Other(&'static str)` at `client/mod.rs:606`; `Other(reason) => write!(f, "error: {}", reason)` at `client/mod.rs:636` — with the empty reason this renders as the bare string `"error: "`.
- **Unused intended variant:** `ClientErrorKind::RefreshError` is declared at `client/mod.rs:602` and rendered at `client/mod.rs:620-622`, but grep shows it is never constructed anywhere in the workspace — the variant designed for exactly this failure was never wired up.
- **Good contrast paths in the same file:** `auth()` failure arm `client/mod.rs:300-322` (serializes `errors` into the `ClientError.source`), and `handle_response` `client/mod.rs:650-677` (`APIResponseError` with the joined error strings as `source`). `refresh()` uses neither.
- **Register's `wasm` `AuthError` claim — NOT VERIFIABLE in current code:** there is no `AuthError` type anywhere in the workspace. The only `AuthError` identifier is the unit variant `ClientErrorKind::AuthError` (`client/mod.rs:601`), whose Display is the static string `"encountered an error authenticating"` (`client/mod.rs:617-619`) — no empty-string wart. The `wasm` module (`src/oxidauth/oxidauth-rs/src/wasm/mod.rs`, `builder.rs`, feature `wasm = ["gloo-storage", "gloo-timers"]` in `oxidauth-rs/Cargo.toml:45`) is a LocalStorage state skeleton with no HTTP layer and no error type of its own (the builder returns `Result<OxidauthClient, String>`). Git history (`git log -S AuthError`) confirms no such type existed pre-layout-move either. Treat the wasm half of the register line as stale; there is nothing separate to fix (see Proposed resolution).
- **Envelope/server side:** `src/xlib/http/src/lib.rs:10-27` (client envelope), `src/oxidauth/oxidauth-kernel/src/error.rs:7-22` (`OxidAuthError`), `src/oxidauth/oxidauth-api/src/server/api/v1/refresh_tokens/exchange.rs:36` (failure handler), `src/oxidauth/oxidauth-services/src/refresh_tokens/exchange_refresh_token.rs:123` (real error copy).

## Problem

When the server rejects a refresh, `Client::refresh()` throws away everything the server said and returns an error that carries no message, no source, and no status:

```rust
// client/mod.rs:419
_ => return Err(ClientError::new(ClientErrorKind::Other(""), None)),
```

`Display` for `Other` is `write!(f, "error: {}", reason)` (`client/mod.rs:636`), so the caller sees the literal string `"error: "` and `err.source().is_none()` — the pinned test asserts exactly this degenerate triple at `client/mod.rs:1076-1078`.

The wire format preserves everything the client discards. The server handler maps any exchange failure to `Response::bad_request().error(err.into_error())` (`oxidauth-api/.../refresh_tokens/exchange.rs:36`), i.e. HTTP 400 with body `{"success": false, "errors": [{...OxidAuthError...}]}`, where each `OxidAuthError` serializes as `{name, display, debug, status_code?, context?, source?}` (`oxidauth-kernel/src/error.rs:7-22`). For an expired token the real payload contains `"refresh token has expired"` (`oxidauth-services/src/refresh_tokens/exchange_refresh_token.rs:123`). The client deserializes this into `oxidauth_http::Response<ExchangeRefreshTokenRes>` — `errors: Option<Vec<serde_json::Value>>` (`xlib/http/src/lib.rs:19`) — so the messages are in memory in the `match` scrutinee, and the `_` arm drops them.

The catch-all arm also swallows two further failure shapes identically:
1. **Non-`2xx` with a well-formed envelope** — the HTTP status is never inspected: `refresh()` calls `.send().await?.json().await?` directly (`client/mod.rs:343-361`) without `Response::status()`. And the envelope cannot carry it back: `Response`'s `status_code` field is `#[serde(skip)]` (`xlib/http/src/lib.rs:25-26`), so on deserialize it is always the default regardless of the server's status line. A 401 (client-key revoked) is indistinguishable from a 400 (token expired) and from a 500.
2. **Malformed success** — `success: true` with `payload: None` also falls into `_`, so a server bug yields the same empty error instead of something like `EmptyPayload`.

## Analysis

- **Mechanism:** `auth()` pattern-matches `Response { success: false, errors: Some(errors), .. }` separately (`client/mod.rs:300-316`): it JSON-serializes `errors` and attaches it as the `ClientError.source` under `Other("failed authenticate response")`. `handle_response` (used by every contract resource wrapper) does the equivalent for all other endpoints: `APIResponseError` with the errors joined into a `String` source (`client/mod.rs:658-669`). `refresh()` — one of only two hand-written raw-call paths — got the `_` arm instead.
- **Type constraint:** `ClientErrorKind::Other(&'static str)` (`client/mod.rs:606`) is `pub` and cannot hold server-generated (owned) text. Any fix that puts server text *into the kind* must change the public enum; a fix that puts it in `ClientError.source` needs no public type change (`source: Option<Box<dyn Error + Send + Sync>>` already, `client/mod.rs:538`).
- **Related failure surface:** the arm above (`client/mod.rs:411-416`, `"failed to validate jwt"`) also carries no source. Note this arm is operationally unreachable against a live server until CLI-2/CLI-3 are fixed: the stale `raw_jwt` bug (register CLI-2 — ticketed as OXA-000028, same function) and the base64-vs-PEM verification bug (`client/mod.rs:371-382`, register CLI-3, pinned at `client/mod.rs:1290`) together mean expiry-driven refresh is already broken end-to-end. **[Review correction 2026-09-30: the "no ticket" claims on this line were stale — CLI-3 = OXA-000029, CLI-5 = OXA-000031, both exist. OXA-000029 gates whether this arm is even reachable end-to-end, and both tickets edit the same `client/mod.rs` test module — sequence them, never parallelize.]**
- **Ordering quirk (context, not this bug):** `refresh()` fetches the public keyset *before* checking the refresh token (`client/mod.rs:332-339`), pinned at `client/mod.rs:1048-1049`; the keyset fetch can itself fail (`Other("unable to fetch public keys")`, `client/mod.rs:188-192`) before the server even sees the refresh request.

## Impact

- **Who:** SDK consumers only — `Client::refresh()` is `pub` (`client/mod.rs:329`) and is the automatic re-auth path for every ordinary API call: `request()` → `authenticate_if_needed()` → `AuthState::Refresh` → `refresh()` (`client/mod.rs:443-463`). When a JWT expires and the refresh token is dead (expired/revoked/rotated-out), a user's next `get_users()`, `can()`, etc. fails with `ClientError` rendering as `"error: "` and an empty source chain. The server binary does not use this code path.
- **What is lost:** the server's message(s) ("refresh token has expired", validation copy, `name`/`debug` fields), the HTTP status (400 vs 401 vs 500 — drives whether the caller should re-login, back off, or page someone), and any `warnings`/`notices` on the envelope.
- **Operational cost:** undiagnosable support tickets ("login dies after N days and says nothing"), and any caller doing error classification/matching on message text gets an empty signal. Once CLI-2/CLI-3 make expiry-driven refresh actually run against live servers, this becomes the *only* feedback on the most common auth failure (expired refresh token).
- **Aggravator:** `RefreshError` exists and renders "encountered an error while refreshing token" but is never constructed — so nothing in the current error taxonomy even tells the caller the failure happened during refresh rather than during login (login uses `Other("failed authenticate response")`).

## Proposed resolution

**Step 1 — non-breaking, matches existing crate convention (`auth()` and `handle_response` already do this):** replace the arm at `client/mod.rs:419` with two arms, preserving server detail in the `source` (no public type change):

```rust
Response { success: false, errors: Some(errors), .. } => {
    return Err(ClientError::new(
        ClientErrorKind::RefreshError,           // finally wired up
        Some(serde_json::to_string(&errors).unwrap_or_else(|_| format!("{errors:?}")).into()),
    ));
},
_ => {
    return Err(ClientError::new(ClientErrorKind::RefreshError, Some("malformed refresh response".into())));
},
```

and, in the same edit, capture the status before `.json()` consumes the response (`client/mod.rs:343-361`) — `let (status, response) = res.error_for_status_ref()...` or simply `let status = res.status(); let response: Response<...> = res.json().await...` — and include it in the source string (e.g. `format!("{status}: {errors}")`), since the envelope's `status_code` is `#[serde(skip)]` and cannot survive deserialization.

**Step 2 — optional typed variant (API break; gate on maintainer appetite):** add `ClientErrorKind::ServerError { status: Option<u16>, errors: Vec<serde_json::Value> }` (or `ServerErrors(String)`) instead of overloading `Other`. This is a breaking change for downstream exhaustive `match`es on the `pub` enum (`client/mod.rs:598-607` has no `#[non_exhaustive]`); consider adding `#[non_exhaustive]` at the same time. Keep `Other(&'static str)` as-is so `client_error_display_is_pinned` and the ~14 in-crate `Other(...)` sites are untouched. A minimum variant that just fills `Other` is not possible without changing `Other` to carry owned text — strictly worse for compat than a new variant.

**wasm parity / Display:** no wasm-side change exists to make — see Locations; the register's `wasm AuthError` is stale. Wasm consumers using the native `Client` under the `wasm` feature inherit the fix automatically. If Step 1 is taken, the `Other` Display arm (`client/mod.rs:636`) needs no change either, because no `Other("")` remains; if the team wants belt-and-braces, make `Display` skip the empty-reason case (`if reason.is_empty() { write!(f, "error") }`) — cosmetic only.

**Pin flips (all in `client/mod.rs` tests):**
- `refresh_failure_envelope_collapses_to_an_empty_error` (`1053-1083`): rewrite `BUG(pinned)` block at `1074-1078` — flip `matches!(err.kind, ClientErrorKind::Other(""))` → `matches!(err.kind, ClientErrorKind::RefreshError)`, replace `format!("{err}") == "error: "` with the new pinned copy, flip `err.source().is_none()` → assert the source contains `"refresh token has expired"`. Keep the state-survival assertions at `1080-1082` (they must stay green).
- `client_error_display_is_pinned` (`1358-1405`): survives unchanged under Step 1; flips only if Step 2 adds a variant (add a case for it) or the Display arm changes.
- `refresh_without_stored_token_fails_after_refetching_keys` (`1036-1051`): unchanged under Step 1 (different arm); flips if the no-token arm is also retyped to `RefreshError` (not proposed here — out of scope, same file, judgment call for the implementer).
- No other assertions on `ClientErrorKind::Other` relate to refresh: the remaining `Other(` test matcher is `auth/authenticate.rs:74` ("unable to deserialize public keys"), a different arm.

## Verification

- Unit: `cargo test -p oxidauth client::tests::refresh_failure_envelope` — rewritten test must now assert the server message reaches `Display`/`source`, still with `received(&server, "POST", REFRESH_PATH)` hit once and prior auth state intact. (Package name is `oxidauth`, not `oxidauth-rs` — `-p oxidauth-rs` matches no package.)
- Unit: `cargo test -p oxidauth client_error_display_is_pinned` (must stay green under Step 1; under Step 2 extend the table at `client/mod.rs:1362-1381`).
- New/extended wiremock case (cheap, same harness): mount `REFRESH_PATH` returning **401** with `{"success":false,"errors":[{"name":"Auth","display":"unauthorized"}]}` and assert the status code appears in the rendered error/source — proves the `#[serde(skip)]` status gap is closed.
- Malformed-envelope case: mount `200` with `{"success":true}` (no payload) and assert a non-empty, distinguishable error.
- Full crate: `cargo test -p oxidauth` (native; wiremock dev-dep is gated `not(target_arch = "wasm32")`, `oxidauth-rs/Cargo.toml:55-56`; package name is `oxidauth`). wasm build sanity only: `cargo build -p oxidauth --features wasm --target wasm32-unknown-unknown` (Step 2 introduces `serde_json::Value` in a public variant — confirm it's already a dependency; it is, `Cargo.toml:29`).
- Consumer-facing check (Step 2, API break): grep downstream for exhaustive `match (err.kind)` on `ClientErrorKind`; in-repo there are none outside `client/mod.rs` and two test-only `matches!` usages (`auth/authenticate.rs:74`, `users/contract.rs:162,183`), neither matching `Other`/`RefreshError`.
