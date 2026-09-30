# OXA-000037 — OAuth2 callback answers HTTP 200 on every failure path (plain-text `ERROR_RESPONSE`, no 4xx/5xx)

**Original ID:** SRV-1 · **Severity:** P2 · **Type:** bug · **Status:** open
**Review tier:** Tier 3 (small precise recipe, multi-file) — ranked #21 of 27 · reviewed 2026-09-30 · DEFERRED (owner decision; claims verified, Phase 1 sound — revisit later) · live effect gated by OXA-000029


## Locations

Register line (`BUGS_AND_NOTES.md:66`) cites `oxidauth-api/src/server/api/v1/auth/oauth2/callback.rs:322`. Paths below are relative to `src/oxidauth/` unless noted.

**Register drift, confirmed:** `:322` is the `BUG(pinned)` comment inside the test module (`callback.rs:322-325`), whose text the register line reproduces. The real defect is the three `ERROR_RESPONSE.into_response()` returns in the handler — `:52`, `:89`, `:101` — plus the constant itself at `:26-27`. The marker text accurately describes the defect; only the pointer is test-side (the register's known pattern).

- Defect sites (all in `oxidauth-api/src/server/api/v1/auth/oauth2/callback.rs`, handler `handle` at `:30-104`):
  - `:44-54` — `serde_json::to_value(&params)` failure arm → `return ERROR_RESPONSE.into_response()` at `:52`. **The register names only two arms; this third one exists too** (serialize of `OAuth2AuthenticateParams { code: String, scope: Option<String>, client_key: Uuid }` — practically unreachable, but it shares the defect).
  - `:83-90` — `gen_redirect_url` failure (`Err(url::ParseError)`) → `return ERROR_RESPONSE.into_response()` at `:89` (the register's "`gen_redirect_url`-failure fallback"; confirmed).
  - `:95-102` — `service.authenticate_or_register(...)` returns `Err(err)` → `ERROR_RESPONSE.into_response()` at `:101` (underlying error only logged `?err` at `:96-99`).
  - `ERROR_RESPONSE: &str` constant at `:26-27`: `"OAuth2 Error - unable to authenticate. Contact support for assistance."`
- Success arm for contrast: `:93` `Redirect::to(location.as_str())` → axum 0.8.9 (`oxidauth-api/Cargo.toml:22`) returns **303 SEE_OTHER, not 302** — pinned by the handler's own tests at `callback.rs:344` and `:361`.
- Route: `GET /v1/auth/oauth2/callback/{client_key}` (`oxidauth-api/src/server/api/v1/auth/oauth2/mod.rs:14`, nested via `.../v1/auth/mod.rs:12`); browser-facing, unauthenticated.
- Extractor rejections (adjacent, *correct status*, plain-text body): `Query<OAuth2AuthenticatePathParams>` at `callback.rs:33` + kernel struct `oxidauth-kernel/src/auth/authenticate_or_register.rs:74-79` (`code: String` required, no `error` field) — missing `code`/`state` and non-uuid `client_key` already yield axum 400s, pinned at `callback.rs:271-315`.
- Pinned tests: `service_failure_maps_to_the_static_error_text` (`callback.rs:317-328`, marker `:322-325`, asserts `StatusCode::OK` + body `== ERROR_RESPONSE` at `:326-327`) and `unbuildable_redirect_url_falls_back_to_the_static_error_text` (`callback.rs:374-386`, asserts `OK` + body at `:384-385`, **no marker** — it will flip silently when fixed).
- Error taxonomy feeding the `Err` arm: `oxidauth-services/src/auth/authenticate_or_register.rs:120-237` — authority lookup `?`/`let-else` `:129-137` (today `RowNotFound` via `Err`, see OXA-000021), state parse `:140-142`, Argon2 state verify `:144-156`, `AuthorityParams`/`OAuth2AuthenticateParams` `try_into()?`, `fetch_profile` → raw reqwest/serde text (OXA-000010), non-register authenticate errors, and `register(...).await?` — **all of them collapse to the identical 200 + static string.**
- `client_base` (the handoff URL the success arm redirects to) exists only inside the *success* response: `AuthenticateOrRegisterResponse.client_base: Url` (`oxidauth-kernel/src/auth/authenticate_or_register.rs:25`), sourced from operator-configured `AuthorityParams.client_base_url` (`oxidauth-services/src/auth/strategies/oauth2/mod.rs:37`, `authenticate_or_register.rs:190,223`). On every error path the handler has only `client_key` — no base to redirect to.
- Contrast inside the same router: sibling `POST /oauth2/redirect` maps service errors to `Response::bad_request().error(err.into_error())` → 400 + JSON envelope (`oxidauth-api/src/server/api/v1/auth/oauth2/redirect.rs:36`; envelope/status mechanics `src/xlib/http/src/lib.rs:63-65,77,146-148`). The callback is the only auth handler that answers failures with an implicit 200.

**Register wording to correct:** "clients/hurl can't distinguish failure" is the *consequence*, not existing coverage — grep over `src/oxidauth/hurl/` for `callback|OAuth2 Error|sso/login` returns **no matches**: the hurl suite never touches the callback route and asserts no 200. The only assertions pinning the wrong status are the two unit tests above.

## Problem

Every failure of `GET /v1/auth/oauth2/callback/{client_key}` — forged/expired state, unknown `client_key`, IdP code-exchange rejection, IdP outage, register failure, malformed authority config — answers **HTTP 200** with the plain-text body `ERROR_RESPONSE`. Axum's `IntoResponse` for `&str` sets 200 + `text/plain; charset=utf-8` and nothing else in the handler overrides the status. A client (browser automation, hurl, e2e suites, monitoring probes) therefore cannot distinguish an SSO failure from a success at the HTTP layer without body-string matching; success and failure differ only in status 303-vs-200 (success *is* distinguishable — but anything following the 303 redirect never re-checks the callback leg, and any tool that inspects the callback response itself sees "OK" for a failed login).

The register's two named arms (`Err` at `:101`, `gen_redirect_url` fallback at `:89`) share the defect exactly as described; the params-serialize arm at `:52` is a third instance it misses.

## Analysis

**Mechanism.** `&'static str` implements axum's `IntoResponse` as `(StatusCode::OK, [("content-type", "text/plain; charset=utf-8")], body)`. None of the three error returns composes a status in, unlike every other handler in `v1/auth`, which funnels errors through the `oxidauth_http::Response` envelope carrying an explicit 400. So the status code silently defaults.

**What feeds the `Err` arm (services `:120-237`).** In order of execution: (1) `FindAuthorityByClientKey` — either `Err(RowNotFound)` today (the `.await?` short-circuit OXA-000021 fixes) or its revived `None → Err("could not find authority by client key")` arm; (2) state string not parseable as a PHC hash → `Err("could not hash state")`; (3) Argon2 verify fail → `Err("Invalid state hash")`; (4) authority `params` or caller `params` fail `try_into()`; (5) `fetch_profile` — `exchange_*_token`/`retrieve_*_profile` failures, which OXA-000010 documents as raw reqwest/serde prose with no status check and no domain type; (6) `AuthenticateUseCase` errors that aren't the `"user authority not found:"` register-fallback trigger; (7) `register(...).await?`. None of these have a typed error at the handler boundary (`BoxedError`, stringly-typed), so **today the handler literally cannot tell "you sent junk" (400) from "your IdP is down" (502) from "your authority is misconfigured" (500)** — and chose not to try, returning 200 for all.

**Why "200 OK" is especially wrong on this route.** It is an unauthenticated `GET` whose URL arrives from a third party (the IdP redirect). Automated fetchers — link prefetchers, chat-link unfurlers, safety scanners, uptime probes — load such URLs and cache the verdict; a 200 says "this resource exists and is fine". It also poisons observability: ingress/CDN metrics and any alert keyed on 4xx/5xx rates will show a 100%-healthy callback route during a total IdP outage or an active login-CSRF probing campaign (OXA-000010 shows states are free to forge, so the callback gets hammered with junk states — every probe scores a 200).

**The IdP-leg asymmetry (adjacent, already 400).** When the *user denies consent at Google/Microsoft*, the IdP redirects to the callback with `?error=access_denied&state=…` and **no `code`** (RFC 6749 §4.1.2.1 in the direction where oxidauth *is* the OAuth client). `OAuth2AuthenticatePathParams` requires `code` (kernel `:76`, no `error` field), so axum's `Query` extractor rejects with a bare-text 400 — pinned at `callback.rs:271-284`. Status-correct, but it's axum's rejection prose, not the JSON envelope the rest of the API uses, and the `error`/`error_description` from the IdP are dropped. Fixing the *user-denied* UX is a design choice, not a defect; noted so this ticket's Phase-2 proposal subsumes it deliberately rather than by accident.

**"Redirect-error per RFC 6749 §4.1.2.1" — applicability.** §4.1.2.1 governs the *authorization server* redirecting back to its registered `redirect_uri` with `?error=&state=`. On this route oxidauth is the OAuth **client** (Google/Microsoft are the AS); the IdP leg is not ours to redirect — we consume its redirect. The downstream hop `client_base/auth/sso/login/{refresh_token}` is oxidauth's *proprietary* handoff to its own SPA (`strategies/oauth2/mod.rs:37-38` documents `client_base_url` as "Client frontend url that callback should redirect the user to"), not an RFC-governed `redirect_uri` — there is no per-client registered-URI table to validate against. So the RFC supplies *shape to borrow*, not a compliance obligation: `3xx` back to the operator-fixed `client_base` with a static `error` code and the echoed `state`, only when that base is actually known. Two hard constraints: (a) when the authority lookup or its `params` parse failed, there is **no trustworthy base** — only the bare-4xx answer is safe; (b) `client_base_url` comes from the operator's authority row (DB), never from query input, so redirecting there introduces no open redirect — but any echoed `state` MUST be appended via `url.query_pairs_mut()` (attacker-controlled string; same encoding discipline `gen_redirect_url:131-143` already uses). Note also the success arm emits 303 (`Redirect::to`), so 303 — not the register/task's "302" — is the house 3xx.

**Ordering dependency.** Precise status classification (400 vs 502 vs 500) is unachievable until OXA-000010 Step 3 lands `OAuth2StateError` / `OAuth2ExchangeError` — until then every `BoxedError` is prose and the honest coarse answer for "the service said no" is 400 (the register's own floor: "proper status (400/502)"). The `client_base`-bearing redirect (Phase 2) additionally needs the service to surface the parsed base on failure, since the handler doesn't do authority lookups.

**Must-not-break constraints:**
- OXA-000004's register-fallback string match (`err.to_string().contains("user authority not found:")`, services `:197-231`) is *inside* the service and stays untouched — this ticket changes only the transport's answer once the service has already decided to fail. But it is the reason "user not linked" must never be reinterpreted at the handler: it isn't an error leg at all.
- OXA-000021's claim that its own fix leaves the callback "browser text unchanged" must remain true after *this* ticket too — the fix here must not leak `RowNotFound`/reqwest prose into the browser body (OXA-000010's Impact §53 says exactly that keeping `ERROR_RESPONSE` as the body is what protects: "any future client that surfaces service errors (see SRV-1) would leak transport internals"). Fix = add a status; do not replace the static text with `err.into_error()` bodies.
- The two 303-success tests (`:330-372`) and the three extractor-400 tests (`:271-315`) stay green untouched.
- No SDK surface: `oxidauth` (dir `oxidauth-rs`) exposes only `oauth2_redirect` (`oxidauth-rs/src/client/auth/oauth2/redirect.rs:16-28`); the callback is browser-only. Repo-wide grep for the body string confirms no in-tree consumer parses it besides the two tests. CLI-1..8 / OXA-000027..000034 (SDK client tickets) are unaffected.

## Impact

- **Who:** every tenant running an OAuth2 authority; every failed SSO login (user denies consent at the IdP → 400-prose today; expired code, revoked IdP secret, replayed state after OXA-000010's fix, unknown client_key, IdP outage, misconfigured `client_base_url`). Operator monitoring; e2e/hurl suites; any embedder automating the flow.
- **Severity P2, not P1:** no auth bypass — the body still says "OAuth2 Error" and no token is handed out; users see the correct message. The damage is *machine-facing*: failure indistinguishable from success except by string-parsing an HTML-invisible plain-text body, zero observability into SSO failure/outage rates via status codes, and scanners/monitors silently "accepting" failed-login URLs as healthy resources.
- **Alerting inversion:** an IdP outage or brute-force state-probing campaign (free states per OXA-000010) moves no dashboard at all — the route is 100% 2xx by construction. Meanwhile nothing can retry/distinguish transport failure vs client error programmatically.
- **Suite coupling:** exactly two in-tree tests bake in the wrong status (`callback.rs:326`, `:384`); no hurl assertion exists to flip, and the fix should *add* coverage (Verification).

## Proposed resolution

**Phase 1 — status codes on the existing body (self-contained, ship first).** Keep `ERROR_RESPONSE` as the browser-visible body verbatim (anti-leak constraint above); only compose statuses:

```rust
// callback.rs :52  (unreachable in practice — serialize of String/Option<String>/Uuid)
return (StatusCode::INTERNAL_SERVER_ERROR, ERROR_RESPONSE).into_response();
// callback.rs :89  (operator misconfig: client_base_url unjoinable — cf. test at :374-386)
return (StatusCode::INTERNAL_SERVER_ERROR, ERROR_RESPONSE).into_response();
// callback.rs :101 (service said no: bad state/authority/params/IdP — honest coarse floor)
return (StatusCode::BAD_REQUEST, ERROR_RESPONSE).into_response();
```

400 is the coarse floor for *all* service errors until Phase 3 (see dependency); it matches the register's "proper status (400/502)" minimum and the project's "not-found is 400" convention (`docs/migration-plan/11-scripts-and-tests.md:112-118`, cited by OXA-000021). Add `use axum::http::StatusCode;` (test module already imports it at `:157`; the product module does not). Compat: the *browser* UX is byte-identical (same text, browsers ignore status on rendered GETs); anything programmatically checking the callback flips from false-success to correct-failure — i.e., only consumers relying on the bug break. No SDK/CLI touchpoint exists.

**Phase 2 (recommended, optional) — spec-shaped redirect-error where the base is known.** To give the SPA a chance to render `?error=` instead of a dead text page: change the AOR service error surface so failures *after* `AuthorityParams` parse carry the parsed `client_base` (e.g., a small domain error enum in `oxidauth-kernel` alongside OXA-000010's Step-3 types holding `{ client_base: Option<Url>, state: Option<String>, kind }`). Handler rule: if a failure yields `Some(client_base)` → `Redirect::to` `{client_base}/auth/sso/error?error=<static code>[&state=<echoed via query_pairs_mut>]`; else → Phase-1 bare 4xx. Static codes only (RFC 6749 §4.1.2.1 vocabulary: `access_denied`, `invalid_request`, `server_error`, `temporarily_unavailable`); never the IdP's or the stored error string; never redirect when the authority row itself didn't resolve/parse. Same mechanism naturally covers "user denied at IdP" *if* the query struct later grows optional `error` fields — that's a UX decision recorded here, not required by this fix.

**Phase 3 — precise statuses (depends on OXA-000010).** Once `OAuth2StateError` / `OAuth2ExchangeError` / `OAuth2ProfileError` exist, downcast at `:95-102`: state/authority/params errors → 400; `ExchangeError::Transport` / IdP 5xx / profile transport → 502; internal (`register` failure, JWT signing, DB errors) → 500. Until then do **not** string-sniff `BoxedError` prose to fake classification — that would pin OXA-000010's raw-transport strings into a second contract.

**Pinned-test flips (marker census: exactly one marker under this file, `:322`; the second pin is unmarked):**

| Site | Today | Flip to |
|---|---|---|
| `callback.rs:317-328` `service_failure_maps_to_the_static_error_text` (marker `:322-325`) | `assert_eq!(status, StatusCode::OK)`; body `== ERROR_RESPONSE` | delete marker comment; status → `BAD_REQUEST` (Phase 2/3: 303+Location to `/auth/sso/error` or classified code); **keep** the body assertion — proves text (not status) is still the human answer and no `err` leaks |
| `callback.rs:374-386` `unbuildable_redirect_url_falls_back_to_the_static_error_text` (no marker) | `StatusCode::OK`; body `== ERROR_RESPONSE` | status → `INTERNAL_SERVER_ERROR`; keep body assertion. Add a `BUG(pinned)`-style comment removal note in the diff so reviewers don't read the unmarked red as unrelated |
| `callback.rs:271-315` (three extractor-400 tests), `:330-372` (two 303-success tests) | correct | unchanged, must stay green |

OXA-000021 needs no coordination edits — its hurl flip is `authenticate.hurl` only; its "callback browser text unchanged" claim survives because this ticket keeps the text. When green, drop `BUGS_AND_NOTES.md:66` (correcting the "hurl" clause as described in §Locations).

## Verification

- **Fails-before/pass-after (in-tree):** extend the two tests above with the new status assertions *before* the product change — `cargo test -p oxidauth-api` (targeted: `cargo test -p oxidauth-api oauth2::callback`) shows exactly `service_failure_…` and `unbuildable_redirect_url_…` red; apply Phase 1 → green, no other callback test moves. Compile-check with `cargo test --no-run -p oxidauth-api -p oxidauth-services` (plain `cargo check` doesn't build tests, repo note T-3 in OXA-000010 §Verification).
- **Live repro (no IdP, no DB seed needed):** against the running stack,
  `curl -si "$BASE/api/v1/auth/oauth2/callback/$(uuidgen)?code=x&state=not-a-phc-hash"`
  — pre-fix: `HTTP/1.1 200` + `text/plain` "OAuth2 Error…"; post-fix: `HTTP/1.1 400`, same body. (Unknown `client_key` errors at the authority lookup *before* any IdP contact — the ordering OXA-000010's tests pin.) Spot-check that a *missing* `code` still returns the extractor 400 and the success arm remains 303 wherever a live sandbox login can be run.
- **New hurl coverage** (the register's "hurl can't distinguish" is fixed by *adding* it — none exists today): `src/oxidauth/hurl/tests/oauth2_callback.hurl`: (1) unknown-key callback → `HTTP/1.1 400` + `[Asserts] textContains "OAuth2 Error"`; (2) missing-`code` callback → `HTTP/1.1 400`. Run via `./src/oxidauth/hurl.sh`; the rest of the suite must not move (grep-verified no other file references this route or string).
- **Phase 2 (if taken):** handler-test with the mock service erroring while yielding a `client_base` → assert 303, `Location` = `{base}/auth/sso/error?error=…&state=` with the state percent-encoded (reuse the encoding assertions' style at `callback.rs:388-409`); mock erroring *without* a base → 400.
- **Phase 3 (deferred to OXA-000010 landing):** downcast-driven 400/502/500 covered by extending `service_failure_maps_to_the_static_error_text` into a table test per error variant; do not schedule before the typed errors exist.
- **Cross-checks:** `cargo test -p oxidauth-services auth::authenticate_or_register` untouched/green (service semantics unchanged); `oxidauth` SDK (`oxidauth-rs`) suite untouched/green — no callback surface (`oxidauth-rs/src/client/auth/oauth2/`).
