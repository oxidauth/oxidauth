# OXA-000010 — AOR `state` verifies a hash of public data: forgeable, replayable forever; exchange failures leak raw reqwest text

**Original ID:** SEC-10 · **Severity:** P2 · **Type:** bug · **Status:** open
**Review tier:** Tier 5 (hard — cross-crate redesign / format migration / IdP flow) · unreviewed


## Locations

Register path (`oxidauth-services/src/auth/authenticate_or_register.rs`) is where validation lives, but minting lives elsewhere — the fix spans both files. Register qualifier "pinned by test, no marker" is confirmed: neither file carries a `BUG(pinned)` marker for the state design (the marker at `redirect.rs:251` pins audit B2.1 / register SRV-7, a different defect).

- Validation: `src/oxidauth/oxidauth-services/src/auth/authenticate_or_register.rs:139-156` — `PasswordHash::new(&params.state)` then `Argon2::default().verify_password(authority.client_key.as_bytes(), ...)`; rejections are `format!` strings (`:141` `"could not hash state"`, `:154` `"Invalid state hash"`).
- Minting: `src/oxidauth/oxidauth-services/src/auth/strategies/oauth2/redirect.rs:59-63` (call site; errors via bare `std::fmt::Error`) and `:102-111` — `hash_client_id`: `argon2(client_key.into_bytes(), SaltString::generate(&mut OsRng))`, appended to the IdP URL as `state` at `:72` (Google) and `:84` (Microsoft).
- HTTP surface (both public): `POST /v1/auth/oauth2/redirect` and `GET /v1/auth/oauth2/callback/{client_key}` — `oxidauth-api/src/server/api/v1/auth/oauth2/mod.rs:13-14`, nested via `.../v1/auth/mod.rs:12`; callback handler `.../oauth2/callback.rs:30-104` passes `auth_response.state` (`:62`) into the use case; error arm `:95-103` logs `?err` and returns the static `ERROR_RESPONSE` (`:26-27`).
- Kernel contracts: `oxidauth-kernel/src/auth/authenticate_or_register.rs` — `state: String` at `:52` (params) and `:78` (`OAuth2AuthenticatePathParams`); trait `:11-18`. DI: `oxidauth-api/src/provider/services.rs:67-77` (AOR), `:81-88` (redirect).
- Exchange/profile calls that surface raw reqwest text: `.../oauth2/google/exchange_token.rs:26-31` and `.../microsoft/exchange_token.rs:25-30` (`.map_err(|err| err.to_string())?` on both `.send()` and `.json()`; **no HTTP status check anywhere**), `.../google/retrieve_profile.rs:20-22` and `.../microsoft/retrieve_profile.rs:20-22` (bare `?` on `reqwest::Error`). Called from `authenticate_or_register.rs:82-104` (`fetch_profile`), invoked at `:165`.
- Domain-error convention to follow: `oxidauth-kernel/src/authorities/mod.rs:159-195` (`AuthorityNotFoundError` — hand-written enum + `Display` + `Error`, boxed into `BoxedError`).

## Problem

The AOR `state` check validates *nothing secret and nothing ephemeral*. Verification is plain argon2 over `client_key` bytes — and `client_key` is a public identifier (it is the request body of `POST /redirect`, the path segment of the callback URL, and client-side config; OXA-000004 documents the same). The hash embeds no server secret, no timestamp, and no nonce, and is never consumed, so:

1. **Any validly-shaped argon2 hash of the authority's `client_key` is accepted** — anyone who knows the public `client_key` can mint unlimited states offline; they never need to call `/redirect`.
2. **States never expire and are never single-use** — the register's "any state hash ever minted is replayable forever" holds: every state ever emitted in a `redirect_url` (and thus captured in browser history, IdP access logs, `Referer`, proxies) stays permanently acceptable.

The second half of the register line is equally confirmed: IdP exchange/profile failures have **no domain error type**. They propagate as `BoxedError` wrapping stringified reqwest errors or `reqwest::Error` itself, and a test *asserts that string* (see pins below). Because none of the four call sites checks the HTTP status first, an IdP 400 with `{"error":"invalid_grant",...}` arrives as a serde "missing field `access_token`" string, and the (unused) `id_token` in `MicrosoftExchangeTokenRes` (`microsoft/mod.rs:26`) is discarded (`Ok(exchange.access_token)`) — so an operator distinguishing expired code vs. revoked secret vs. IdP outage reads raw transport prose.

**Register drift to correct:** "no salt" is literally false. Minting uses a fresh random salt per hash (`redirect.rs:103`, `SaltString::generate(&mut OsRng)`) and a test enforces it (`redirect.rs:389` "each state hash must use a fresh salt"). The random salt is *irrelevant*, not absent: PHC embeds it in the state string, and against public, fully-known input it adds no binding. The substantive claim — no server-secret binding, no expiry binding → forge and replay forever — is exactly right. Also note the register cites only `authenticate_or_register.rs`; the minting half lives in `strategies/oauth2/redirect.rs`.

## Analysis

**What `state` is for here, and why this design achieves none of it.** In the classic authorization-code flow, `state` binds the flow *initiator's browser session* to the callback, so a victim can't be talked into completing someone else's IdP flow (login CSRF, RFC 6749 §10.12). In oxidauth's shape — the *server* mints state at `POST /redirect` and the *server* verifies it at callback, with no session record between — a correct state design can at least provide: (a) proof the callback URL originated from this server, (b) freshness (short TTL), (c) single-use. `Argon2(client_key)` provides none of the three: (a) is vacuous because the input is public, (b) and (c) are simply absent. What it *does* provide is audit theater: the code comment `// VERIFY THE STATE HASH VALUE` and the green tests (`"state must be an argon2 hash of the authority client key"` at `redirect.rs:218`) make the callback look state-protected.

**Argon2 is the wrong primitive twice over.** A slow KDF is for low-entropy *secrets*; `client_key` is a public UUID, so the hashing buys nothing an attacker can't recompute. And running it over *untrusted input* is a hazard: `Argon2::verify_password` honors the `m`/`t`/`p` parameters embedded in the attacker-supplied PHC string, on an **unauthenticated GET** (`callback.rs` route). The `password-hash` ecosystem documents cost-policy checks as the caller's responsibility; nothing here rejects out-of-profile parameters, so a crafted `$argon2id$...m=4194304,t=4294967295...` string aims multi-gigabyte allocations at the server per request. (I did not measure actual accepted limits in the pinned `argon2` crate version — `[INFERENCE]` from documented KDF-verify-over-untrusted-input semantics; the fix below removes the exposure entirely by never parsing attacker strings as PHC.)

**Blast radius vs. SEC-4.** This is not the auth bypass — a callback still requires a fresh, single-use `code` that only the real IdP will exchange (with `oauth2_secret` + `redirect_uri` bound at `exchange_*_token`). That's why it's P2, not P1: state's role is *defense-in-depth and CSRF hygiene*, and OXA-000004 (SEC-4) is the bypass via `POST /v1/auth/authenticate` that never touches state at all. Until OXA-000004 lands, state is the *only* gate on the callback route, and today it's free to satisfy. Concretely, the login-CSRF scenario works end-to-end today: attacker runs its own Google flow, harvests a callback URL (its `code` + any state of that authority — self-minted or years old), tricks a victim into loading `GET /v1/auth/oauth2/callback/{client_key}?code=...&state=...`; the server exchanges the attacker's code, authenticates/registers the *attacker's* account, and redirects the victim's browser to `client_base` with the attacker's `refresh_token`/JWT (`callback.rs:67-93`, `gen_redirect_url` `:106-146`) — victim is now session-fixed into the attacker's account.

**Related code-path smells found in passing (same functions, fix alongside):** AOR re-implements authority-missing as `format!("could not find authority by client key")` (`:136`) while `redirect.rs:56` uses the proper `AuthorityNotFoundError::client_key`; the state-mint failure `Box`es `std::fmt::Error` (`redirect.rs:60-62`), an error type carrying no information.

**Pins (all unmarked — flip when fixing, per the register's contract):**

| Site | Assertion | Flip to |
|---|---|---|
| `authenticate_or_register.rs:311-318` (fixture `state_hash` + comment "the state param must be an argon2 PHC hash…") and `:889-891` (`valid_state`) | design pin; `valid_state()` feeds every happy-path test (`:899,951,976,1001,1056,1106,1129`) | replace with opaque-nonce states from a mocked state-consumer repo; new: unknown/expired/consumed states error |
| `:915-928` `unparseable_state_is_rejected_before_any_token_exchange`, text pin `:926` `"could not hash state"` | malformed state → error, IdP untouched | keep behavior; match the typed state error, drop substring |
| `:930-943` `state_hashing_a_different_secret_is_rejected`, text pin `:941` `"Invalid state hash"` | wrong-secret argon2 hash rejected | rewrite as "state issued for another client_key / not in the store → rejected"; keep the `idp.requests().is_empty()` invariant (`:942`) |
| `:963-991` `exchange_failure_propagates_before_any_database_work`, text pin `:981-986` `.contains("error sending request")` | **pins raw reqwest text as the error contract** | assert domain error (`OAuth2ExchangeError::Transport`, Step 3 below); keep the "authenticate must never start" log assertion `:987-991` |
| `redirect.rs:211-219` `assert_state_verifies_client_key` helper | "state must be an argon2 hash of the authority client key" | delete helper |
| `redirect.rs:271-272` (google test) `state.starts_with("$argon2")` + verify; `:315` (microsoft test `:283`) | minted state is a client_key hash | assert opaque random nonce + state-store row (mock); leave the `:251` B2.1/SRV-7 marker alone |
| `redirect.rs:382-404` `hash_client_id_is_a_verifiable_random_salted_hash` | "each state hash must use a fresh salt", "state must be bound to the specific client key" | delete with `hash_client_id`; replace with freshness/single-use tests (Verification) |

## Impact

- **Who:** every tenant with an OAuth2 authority (Google/Microsoft flavors); every user logging in through the callback route.
- **What:** the anti-CSRF/replay layer of the OAuth2 login flow is inert — states are forgeable by anyone knowing the public `client_key`, capturable from URLs/logs, valid forever, and reusable. Direct exploitation still requires a real IdP `code`, so the realistic attacks are login CSRF / session fixation (above) and, until SEC-4 is fixed, strengthening an attacker's callback-route footing. The attacker-chosen-parameters verification cost is an unauthenticated resource-DoS handle.
- **Ops/UX:** exchange failures (expired code, revoked IdP secret, wrong `redirect_uri`, IdP down) are indistinguishable in machine-consumable form — only free-text reqwest/serde prose reaches `BoxedError`; the browser only ever sees the static `ERROR_RESPONSE` (`callback.rs:26-27`), so triage depends on log archaeology, and any future client that surfaces service errors (see SRV-1) would leak transport internals.
- **False assurance:** anyone auditing "is OAuth state validated?" gets yes from code comments and green tests; SEC-11's jwt findings show this project's trust boundaries deserve real, not decorative, checks.

## Proposed resolution

OXA-000004 Step 3 already proposes "per-flow single-use nonce; DB table in `oxidauth-postgres`" as its Step 3. **This ticket owns that fix**; OXA-000004 should cross-reference OXA-000010 rather than duplicate it.

**Step 1 — state store (schema + repository).** New migration in `src/oxidauth/oxidauth-postgres/migrations/` (timestamp-prefixed, alongside `20221019…` files):

```sql
CREATE TABLE oauth_states (
  state_hash BYTEA PRIMARY KEY,      -- sha256(state string); raw state never stored
  client_key UUID NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL
);
```

No FK on `client_key` needed (states die in minutes; adding an FK would couple authority deletes to in-flight states). Follow T-6 conventions: `oxidauth-postgres/src/oauth_states/{insert_oauth_state,consume_oauth_state}/`, query traits in `oxidauth-repository/src/oauth_states/` (`InsertOAuthStateQuery`, `ConsumeOAuthStateQuery`), wired in `oxidauth-api/src/provider/services.rs` next to the existing `:81-88` block.

**Step 2 — mint and consume.** In `redirect.rs`: replace `hash_client_id` (`:102-111`) with a 128-bit `OsRng` nonce rendered base64url; insert `(sha256(state), client_key, now + 10 min)` — best-effort `DELETE FROM oauth_states WHERE expires_at < now()` on the same call as lazy GC, no cron. In `authenticate_or_register.rs`: replace the `:139-156` argon2 block with an atomic consume — `DELETE FROM oauth_states WHERE state_hash = $1 AND client_key = $2 AND expires_at > now() RETURNING 1`; 0 rows → typed state error, still *before* any IdP contact (preserve the ordering all three state tests assert). Argon2, `PasswordHash`, `SaltString` imports go away from both files. High-entropy nonce + lookup means SHA-256 is the correct hash — no slow-KDF argument survives; this also closes the attacker-chosen-parameters exposure. A DB store (over an HMAC self-contained token) is chosen because it gives true single-use, adds no secret-management plumbing to `bootstrap`, and matches OXA-000004's proposal; a keyed-HMAC-with-`exp` variant stays open as the stateless alternative if a deployment ever can't write per-login rows.

**Step 3 — domain errors.** In `oxidauth-kernel` (follow `AuthorityNotFoundError`'s shape, `authorities/mod.rs:159-195`):

```rust
pub enum OAuth2ExchangeError {           // + Display + Error
    Transport { client_key, source: reqwest::Error },          // connect/TLS failures
    Rejected { client_key, status, oauth_error, description }, // non-2xx, RFC 6749 §5.2 body parsed
    InvalidResponse { client_key, reason },                    // 2xx but undecodable / missing access_token
}
pub enum OAuth2StateError { Malformed, Unknown, Expired }      // or collapse Unknown/Expired into one "invalid" to avoid an oracle — see below
```

In both `exchange_*_token.rs`: drop `.map_err(|err| err.to_string())`, `send()`, check `response.status()` first, parse `{"error","error_description"}` on 4xx → `Rejected`, decode on 2xx → `InvalidResponse`/success; connect errors → `Transport` (keep `source` for logs). Same shape for both `retrieve_*_profile.rs` (`OAuth2ProfileError`). In AOR, replace the three `format!` errors (`:136`, `:141`, `:154`) with `AuthorityNotFoundError::client_key` (matching `redirect.rs:56`) and `OAuth2StateError`; do not distinguish Unknown vs Expired *to the browser*, but the typed variant lets the `error!` log record the difference. Compat: `state` stays an opaque `String` on every wire type (`kernel:52`, `:78`) — nothing in this repo parses its contents except mint/verify (verified by grep: no `oxidauth-rs`, `oxidauth-http`, `oxidauth-api`, or hurl references); the SPA only passes `redirect_url` to the browser and never sees `state` again. Frontends outside this repo therefore need no change. Deploy-window note: argon2-format states issued pre-deploy fail verification → users mid-flow retry once; do **not** dual-read the old format — rejecting every previously minted state is the point of the fix.

## Verification

- New regression tests in `oxidauth-services/src/auth/authenticate_or_register.rs` (fails-before/pass-after, against the existing fake-IdP harness): (a) the same valid state accepted twice → second call errors and the IdP is contacted exactly once across both; (b) state with expired `expires_at` (mock state repo returns stale row / inject short TTL) rejected with `OAuth2StateError::Expired`-shaped error, `idp.requests().is_empty()`; (c) state stored for authority A rejected on authority B. Flip the four AOR pins per the table; run with `cargo test -p oxidauth-services auth::authenticate_or_register`.
- `cargo test -p oxidauth-services auth::strategies::oauth2::redirect` — after flipping the redirect pins: states unique per call, row inserted with `client_key` + future `expires_at`; the `:251` B2.1 pin stays green untouched.
- Domain error: extend `exchange_failure_propagates_before_any_database_work` to `matches!(err.downcast_ref::<OAuth2ExchangeError>(), Some(Transport { .. }))` for the dead-port case, and add a fake-IdP 400-`invalid_grant` case asserting `Rejected { status: 400, oauth_error: Some("invalid_grant") }` — proves the status-check + §5.2 parse, not the old prose.
- `cargo test -p oxidauth-postgres oauth_states` — insert/consume repo tests on the `#[sqlx::test(migrator)]` harness (local Postgres per `oxidauth-postgres/database_test.sh`); consume is atomic under concurrent identical calls.
- `cargo test -p oxidauth-api` — callback handler tests (mock service) stay green; `cargo check --workspace` won't compile tests (T-3), so use `cargo test --no-run`.
- No hurl or `oxidauth-rs` changes: neither references `state` (grep-verified). Live Google/Microsoft round-trip remains a manual sandbox step — same caveat as OXA-000004, since CI mocks the IdP.
