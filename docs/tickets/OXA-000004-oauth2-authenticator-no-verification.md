# OXA-000004 — OAuth2 authenticator verifies nothing; generic /auth/authenticate is an auth-bypass backdoor

**Original ID:** SEC-4 · **Severity:** P1 · **Type:** bug · **Status:** open
**Review tier:** Tier 5 (hard — cross-crate redesign / format migration / IdP flow) · unreviewed


## Locations

Register path is relative to the `src/oxidauth/` cargo workspace. The cited line `:113` has drifted: it now lands on the `BUG(pinned)` comment in the test module, whose text the register line reproduces verbatim. The buggy code is the `authenticate` body at lines 30–37.

- `src/oxidauth/oxidauth-services/src/auth/strategies/oauth2/authenticator.rs:27-38` — `impl Authenticator for OAuth2`; `authenticate(_authenticate_params, _authority, _user_authority)` returns `Ok(())`, ignoring every argument. Pin comment at `:113-116`; pinned test at `:111-132`.
- `src/oxidauth/oxidauth-services/src/auth/authenticate.rs` — generic pipeline: `AuthenticateUseCase::authenticate` (`:103-276`); `build_authenticator` dispatches `Oauth2 => oauth2::authenticator::new` with no strategy gating (`:282-292`).
- `src/oxidauth/oxidauth-api/src/server/api/v1/auth/authenticate.rs` + `.../auth/mod.rs:14` + `.../v1/mod.rs:21` — generic `POST /v1/auth/authenticate` route, mounted and live.
- `src/oxidauth/oxidauth-kernel/src/auth/authenticate.rs:24-27` — `AuthenticateParams { client_key: Uuid, params: JsonValue }` (arbitrary caller JSON); re-exported as `AuthenticateReq` (`oxidauth-http/src/auth/authenticate.rs:5`).
- `src/oxidauth/oxidauth-services/src/auth/strategies/oauth2/user_identifier_from_request.rs:9-13` — returns the email straight out of caller-controlled params.
- Legitimate flow (for contrast): `oxidauth-api/src/server/api/v1/auth/oauth2/callback.rs:30-104` → `oxidauth-services/src/auth/authenticate_or_register.rs:125-236` (`AuthenticateOrRegisterUseCase`): state verification `:139-155`, code exchange + profile fetch `fetch_profile` `:82-104`, then it calls the generic `AuthenticateUseCase` at `:177-183`, register fallback `:197-231`.
- State generation: `src/oxidauth/oxidauth-services/src/auth/strategies/oauth2/redirect.rs:59,102-111` — `state = Argon2(client_key bytes, random salt)`.
- IdP exchanges: `.../oauth2/google/exchange_token.rs`, `.../oauth2/microsoft/exchange_token.rs` (Microsoft `id_token` is received in `MicrosoftExchangeTokenRes` but discarded — only `access_token` is returned).
- Design contrast: `oxidauth-services/src/auth/strategies/username_password/authenticator.rs:38-63` actually verifies a password hash there.

## Problem

The `Authenticator::authenticate` hook (`oxidauth-kernel/src/auth/mod.rs:37-44`) *is* oxidauth's trust boundary: `AuthenticateUseCase` calls it between "user identified" and "JWT + refresh token minted" (`authenticate.rs:130-132` then `:134-275`). The username_password strategy honors that boundary (password verify). The OAuth2 strategy implements it as `Ok(())`:

```rust
async fn authenticate(&self, _authenticate_params, _authority, _user_authority) -> Result<(), BoxedError> {
    Ok(())
}
```

It verifies nothing that reaches it: no proof the authorization `code` was ever exchanged at the IdP (the params it defines, `AuthenticateParams { email }`, don't even carry a code), no signature/nonce/aud check (Microsoft's `id_token` is thrown away), and no consistency check between params and `user_authority.user_identifier`. Its doc-comment-free design assumes "the caller already completed the IdP code exchange" — but nothing enforces that the caller is `AuthenticateOrRegisterUseCase`.

## Analysis

**The assumption is false at an HTTP boundary.** The generic `POST /v1/auth/authenticate` accepts `AuthenticateReq = { client_key, params: <arbitrary JSON> }` and calls `AuthenticateUseCase` directly, with no strategy gate. For an authority whose strategy is `Oauth2`, the pipeline does:

1. `user_identifier_from_request` → returns `params["email"]` verbatim (attacker-chosen).
2. `SelectUserAuthoritiesByAuthorityIdAndUserIdentifier` → resolves the *victim's* `user_authority` row by that email.
3. `authenticator.authenticate(...)` → `Ok(())`, unconditionally.
4. Mint JWT with the user's full permission-tree entitlements and insert a refresh token.

So `POST /v1/auth/authenticate {"client_key": "<oauth2 authority client_key>", "params": {"email": "victim@corp.com"}}` returns a full-session JWT — no IdP, no code, no state. The callback flow at `authenticate_or_register.rs:177-183` is the *intended* caller and does the exchange upstream, which is exactly why the no-op "works" for real logins and was mistaken for verification.

**Why each "check" doesn't cover this route:**
- *State check* (`authenticate_or_register.rs:139-155`) lives in the AOR use case, not the authenticator; the generic route never runs it.
- *Code exchange* (`fetch_profile`, `:82-104`) likewise lives in AOR. Codes are single-use at the IdP and `redirect_uri`/`client_secret` are sent, so the exchange *is* real verification — but it only gates browsers arriving via `GET /v1/auth/oauth2/callback/{client_key}`, never `POST /v1/auth/authenticate`.
- *Profile→identifier binding* is also upstream: AOR overwrites caller params with `{ email: profile.email }` (IdP-derived) at `:168-175` before calling the use case. The safety of the whole design rests on "email came from the IdP", and the authenticator is the one component that can't tell.

**Adjacent weaknesses (verify, don't silently expand scope):**
- The state check itself is weak: `state = Argon2(client_key)` with a fresh random salt verifies *anything* that ever saw the output of `POST /v1/auth/oauth2/redirect` (anyone who knows `client_key` can mint unlimited valid states; states are never consumed → replayable forever; no browser-session binding → classic CSRF-into-callback still works even on the "verified" path). Tracked here as design context for the nonce work below; the redirect-URL contents pin at `redirect.rs:251` is a separate audit item (B2.1).
- Microsoft `id_token` signature/`aud`/`nonce` are never validated; identity rests solely on the Graph `/me` round-trip with the exchanged access token (Google is the same pattern). Acceptable if the exchange itself is guaranteed — which is precisely what this ticket must guarantee.

**Register drift:** the real fix site is `authenticator.rs:30-37` (current tree); `:113` points at the pin. No git history available to date the drift.

## Impact

- **Who:** every tenant with an enabled OAuth2 authority; every end user with a `user_authorities` row under it.
- **What:** unauthenticated remote account takeover — one POST returns a signed JWT with the victim's full entitlements plus a persisted refresh token. No rate limiting or audit distinguishes it from a normal login; JWTs are indistinguishable from legitimate ones.
- **Prerequisites:** the authority's `client_key` (a UUID, but public by design — it appears in every callback URL, the redirect request body, and client-side config) and the victim's email (enumerable, trivial for targeted attacks). A single leaked `client_key` is a permanent, undetectable backdoor to every user of that authority.
- **2FA interaction:** with authority-level TOTP enabled, the step-up JWT only carries `TOTP_VALIDATE_PERMISSION` and the code webhook goes to the victim — so the bypass degrades to "attacker triggers victim's OTP" instead of instant session. Authorities with `TotpSettings::Disabled` (the common default) are fully instant-compromised.
- The callback path itself is *not* currently exploitable for email spoofing (AOR derives email from the IdP profile before invoking the pipeline), so no data loss is implied beyond what an attacker does with the stolen sessions.

## Proposed resolution

**Step 1 — contain at the transport (small, ship first).** Gate the generic authenticate path to non-OAuth2 strategies. It cannot go inside `AuthenticateUseCase` (AOR calls it in-process at `authenticate_or_register.rs:177-183`, and that call must keep working), so:
- In `oxidauth-api/src/server/api/v1/auth/authenticate.rs`, after resolving the authority by `client_key`, reject `AuthorityStrategy::Oauth2` (and `SingleUseToken`, which `build_authenticator` already `unimplemented!()`-panics on — return an error instead of panicking) with a 4xx error like `"oauth2 authorities authenticate via GET /v1/auth/oauth2/callback/{client_key}"`.
- To keep any future transport from reintroducing the hole, prefer adding a capability to the kernel trait, e.g. `Authenticator::allows_generic_entry(&self) -> bool` (default `true`, overridden to `false` by the OAuth2 impl), checked by the handler.
- Compat: no legitimate caller speaks generic-authenticate to an OAuth2 authority today — `oxidauth-rs` exposes only the redirect helper for OAuth2 (`src/oxidauth/oxidauth-rs/src/client/auth/oauth2/`), login is a browser redirect. A changelog note suffices; the only "callers" broken by this are ones exploiting the bug.

**Step 2 — make the strategy actually verify (structural fix).** The root tension: the OAuth2 identifier (email) exists only *after* the code exchange, but `AuthenticateUseCase`'s pipeline order is identifier → user_authority → authenticate, and IdP codes are single-use so the exchange may not happen twice. Move the exchange *into* the strategy and give the pipeline one resolution step that can do both:
- Change the OAuth2 `AuthenticateParams` from `{ email }` to callback material `{ code, scope }`.
- Have the OAuth2 `user_identifier_from_request` perform `exchange_*_token` + `retrieve_*_profile` and return the IdP-verified email — and make `authenticator.authenticate` verify the exchange actually happened. If threading the verified profile through the existing two-call interface is impractical (it returns only `String`), extend the kernel trait: e.g. `fn resolve_identity(&self, params, authority) -> (String, VerifiedProfile)` as the single hook, with `username_password` keeping today's identifier-then-verify shape. Then `AuthenticateOrRegisterUseCase` shrinks to state check + delegate + register fallback and passes the raw `OAuth2AuthenticateParams` through instead of pre-building `{ email }` at `:168-175`.
- `authenticate` must then fail on anything that isn't a completed exchange for *this* authority (params bound to `user_authority.user_identifier` derived from the verified profile, not the caller).

**Step 3 — fix the callback trust the exchange relies on.** Replace `state = Argon2(client_key)` (`redirect.rs:102-111`) with a per-flow single-use nonce: store `client_key → hash(nonce)` with short TTL (DB table in `oxidauth-postgres`, or existing session store), consume on callback; verify+consume in the AOR path (or wherever Step 2 moves it). Additionally, for both flavors validate the ID token (`iss`, `aud = oauth2_id`, nonce) instead of discarding `id_token` (`microsoft/mod.rs:26`), so identity proof doesn't depend solely on the `/me` round-trip.

**Must-not-break invariants:** the register-fallback path recognizes the typed `UserAuthorityNotFoundError` from the user_authority query via `err.downcast_ref::<UserAuthorityNotFoundError>()` (`authenticate_or_register.rs:199-202`; error defined in `oxidauth-kernel/src/user_authorities/mod.rs`, mapped from `RowNotFound` by `select_user_authority_by_authority_id_and_user_identifier` — since OXA-000025, 2026-09-30; the old `"user authority not found:"` string match is gone) — keep that error flowing from the user_authority query, not the authenticator. The email-as-`user_identifier` keying pinned in `user_authority_from_request.rs` (`"pinned: the callback path identifies by EMAIL"`) stays; only the *provenance* of the email changes from caller-claimed to IdP-derived.

**Pinned tests to flip (`grep -rn 'BUG(pinned)' src --include='*.rs'`, oauth2-scoped):**
- `oxidauth-services/src/auth/strategies/oauth2/authenticator.rs:113` (`authenticate_accepts_anything_after_the_idp_exchange_succeeded`, audit B2.2): replace with assertions that junk params (`{}`), non-objects, missing/expired/forged code material all **error** (Step 2).
- Same file `:134-154` (`authenticate_params_extract_the_email_or_error`) and `user_identifier_from_request.rs:25-61`: caller-supplied-email extraction disappears with the new params shape — delete/rewrite, don't re-pin.
- Out of scope here, same subsystem (separate tickets): `callback.rs:322` (HTTP 200 on failure), `redirect.rs:251` (B2.1 redirect URL contents), `microsoft/mod.rs:145` (null `mail`).

## Verification

- Regression test for the bypass (fails before, passes after): seed an OAuth2-strategy authority + linked user, then `POST /v1/auth/authenticate {"client_key": ..., "params": {"email": "<victim>"}}` and assert 4xx / no JWT — as an `oxidauth-api` handler test plus (preferred) a hurl/e2e hit against the running server; today the same call returns a JWT.
- `cargo test -p oxidauth-services auth::strategies::oauth2` — flipped B2.2 pin: unexchanged/forged params must error; mocked-successful-exchange params must succeed.
- `cargo test -p oxidauth-services auth::authenticate_or_register` — callback path end-to-end against mocked IdP (existing tests already mock exchange/profile): success, register-fallback-on-`UserAuthorityNotFoundError` (typed downcast since OXA-000025), and bad-state rejection all stay green; new: state can be consumed only once (Step 3).
- `cargo test -p oxidauth-services auth::authenticate` and `cargo test -p oxidauth-api` — generic route unchanged for username_password authorities.
- Real Google/Microsoft round-trip can't run in CI (external IdP, single-use codes): the exchange/profile calls are mocked in tests, so live-sandbox verification is a manual step — record it as done against one configured authority before deploy.
