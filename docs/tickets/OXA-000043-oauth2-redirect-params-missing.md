# OXA-000043 — OAuth2 authorization URL relies on unvalidated pre-baked client_id/scope/redirect_uri

**Original ID:** SRV-7 · **Severity:** P2 · **Type:** bug · **Status:** open
**Review tier:** Tier 4 (moderate — multi-file recipes, pinned-test flips) · reviewed 2026-09-30 · DEFERRED (owner decision; structural gap verified intact at HEAD — `redirect.rs` appends only flavor pairs, typed `oauth2_id`/`scopes`/`redirect_uri` read only at token exchange, fixture proves baked-vs-field scope divergence, `ParseOauth2RedirectUrlError` has zero construction sites, no validation at write or read. Recipe sound: strip+recompose from typed fields, wire the dead error as key-drift detector (catches OXA-000003/000016 class), flip pin to field-sourced, OAUTH.md fixes — revisit as first OAuth2-family item; if it lands, pair or same-PR w/ deferred 000031 so new 400s surface as ClientError)


## Locations

- Register line: `BUGS_AND_NOTES.md:72` (SRV-7), citing `oxidauth-services/src/auth/strategies/oauth2/redirect.rs:251`.
- **Marker drift confirmed.** `redirect.rs:251-254` is a `BUG(pinned)` comment inside the Google redirect test, not the defect. The actual construction is `redirect.rs:64-96` (match on flavor): Google arm `:65-76`, Microsoft arm `:77-95`. Pinned assertion at `:255-269`.
- `src/oxidauth/oxidauth-services/src/auth/strategies/oauth2/redirect.rs:45-99` — `Oauth2RedirectUseCase::oauth2_redirect` (state mint call `:59-62`, `hash_client_id` `:102-111`).
- `src/oxidauth/oxidauth-services/src/auth/strategies/oauth2/mod.rs:27-40` — `AuthorityParams` (`redirect_url` `:36`, `client_base_url` `:37`, `redirect_uri` `:39`, `oauth2_id` `:30`, `scopes` `:34`); lazy `TryFrom<JsonValue>` deserialization `:72-80`; test fixtures `:92-110`.
- Field consumers (exchange side): `google/exchange_token.rs:13-20` (sends `oauth2_id`, `redirect_uri`; **no** scope), `microsoft/exchange_token.rs:13-19` (sends `scope: &params.scopes`, `oauth2_id`, `redirect_uri`).
- Write path: `oxidauth-services/src/authorities/create_authority.rs:81,121-122,145` and `update_authority.rs:150,245` — `params` is an opaque `serde_json::Value` stored verbatim; no oauth2-shape or content validation at write time.
- API surface: `oxidauth-api/src/server/api/v1/auth/oauth2/redirect.rs:9-38` (handler), routes `oauth2/mod.rs:13-14` (`POST /v1/auth/oauth2/redirect` public, `GET /v1/auth/oauth2/callback/{client_key}`); request contract `oxidauth-kernel/src/auth/oauth2/redirect.rs:20-24` (`{client_key, email}` only), unused `ParseOauth2RedirectUrlError` `:31-44`.
- Convention doc: `docs/OAUTH.md:80-90` — `redirect_url` example at `:88` bakes `client_id`, `redirect_uri`, `scope` into the query string; step-3 flow text `:15`; step-8 route text `:20` (stale: says `/auth/oauth2/authenticate/:authority_client_key`; actual route is `/callback/{client_key}`).

## Problem

`oauth2_redirect` loads the authority, deserializes `AuthorityParams`, and appends **only** flavor-specific pairs to the stored `params.redirect_url`:

- Google (`:68-75`): `response_type=code`, `include_granted_scopes=true`, `state=<hash_client_id(client_key)>`.
- Microsoft (`:80-92`): `response_type=code`, `response_mode=query`, `state=...`, plus `login_hint` when `params.email` is `Some`.

It never reads `oauth2_id`, `scopes`, or `redirect_uri`. RFC 6749 §4.1.1 marks `response_type` and **`client_id` REQUIRED** in every authorization request (`redirect_uri` OPTIONAL per §3.1.2, `scope` OPTIONAL, `state` RECOMMENDED); the identity trio must instead already sit in the stored URL's query string, per the convention codified in `docs/OAUTH.md:88` and in the test fixture (`mod.rs:100-106`). The audit's B2.1 expectation ("append client_id/scope/redirect_uri here") therefore describes a *design convention*, not missing code — but the convention is enforced **nowhere**: no validation at write time (`create_authority.rs`/`update_authority.rs` store `params` opaquely), and no validation at read time (the use case appends blindly and returns the URL). The result is a latent-defect surface: any authority whose baked query is missing, partial, duplicated, or *inconsistent with the dedicated fields* is accepted and served, and the failure only detonates after the end user has completed a full IdP round trip — or never detonates visibly at all.

The fixture itself proves the enforcement gap: it bakes `scope=openid%20email` (`mod.rs:104`) while the `scopes` field says `"openid email profile"` (`mod.rs:99`). The test suite passes today *because nothing compares the two*.

## Analysis

**Name collision to keep straight.** The post-login redirect builder `gen_redirect_url` in `oxidauth-api/src/server/api/v1/auth/oauth2/callback.rs:106` (redirect to `client_base_url` with tokens) is a *different* URL from the one in this ticket; its error-status handling is OXA-000037. This ticket concerns only the upstream IdP **authorization request URL**.

**Two sources of truth, one-way divergence.** The identity of an oauth2 authority lives in two places: the typed fields (`oauth2_id`, `scopes`, `redirect_uri`) and the raw query string baked inside `redirect_url`. Consumers read only one side each:

| Param | Authorization request (`redirect.rs`) | Token exchange (`exchange_token.rs`) | Consequence of drift |
|---|---|---|---|
| `client_id` | baked value (if any) | always sends `oauth2_id` field | Missing baked value → IdP rejects (REQUIRED, §4.1.1); user is stranded on the IdP's own error page (RFC §4.1.2.1: server MUST NOT redirect on invalid/missing client id), callback never fires, login silently dead. |
| `redirect_uri` | baked value (if any) | **always** sends `redirect_uri` field (both flavors) | RFC §4.1.3: if `redirect_uri` was in the authorization request, the token request must send it and values "MUST be identical" (Google/Microsoft enforce this). Baked ≠ field → `redirect_uri_mismatch`/`invalid_grant` *after* the user has fully authenticated. If not baked at all: provider-dependent — Google tolerates omission only when exactly one redirect URI is registered at the IdP and errors "Missing parameter: redirect_uri" otherwise [provider behavior, not verified in this repo]. |
| `scope` | baked value (if any) | Microsoft sends `scopes` field at the token endpoint (`microsoft/exchange_token.rs:14`); Google never sends scope anywhere | Not baked → Google consent screen requests nothing; token exchanges "succeed" with zero scopes and the profile fetch fails or returns degraded identity (opaque downstream failure). The `scopes` field is read by *no code at all* on the Google path — it is documentation-grade fiction unless manually mirrored into `redirect_url`. |
| duplicates | n/a | n/a | `query_pairs_mut().append_pair` (`:69-72`, `:81-84`) never dedupes. An operator who bakes `response_type` too, or edits the URL twice, emits repeated keys → `invalid_request` ("includes a parameter more than once", §4.1.2.1). |

**Interaction with OXA-000003 (client_key regeneration).** The `redirect_uri` is this server's own callback and embeds the authority's `client_key` in its path (`OAUTH.md:87`: `/api/v1/auth/oauth2/callback/<client_key>`; route `oauth2/mod.rs:14`). OXA-000003 shows `PUT /authorities` regenerates `client_key` when the field is omitted; OXA-000016 covers null-overwrite of params on update. After either, the *baked* callback URI inside `redirect_url` (and the `redirect_uri` field, which nobody re-syncs) points at a route that no longer exists: the IdP redirects the user to a 404. Nothing in the codebase detects this at composition time.

**Why this is not currently an open redirect — and how a careless fix would create one.** Today the caller controls nothing: `Oauth2RedirectParams` is `{client_key, email}` (`kernel:20-24`), and `redirect_url` is entirely operator-written; the server merely echoes operator data. If a fix "composes the URL from parts" and any part becomes caller- or request-controlled (e.g., per-request redirect overrides), RFC 6749 §3.1.2.3 requires the authorization endpoint side (here: the IdP) to match `redirect_uri` against the registered set, and §3.1.2.2/§10.15 warn that skipping registration matching is precisely what turns the endpoint into an open redirector. Our composition side must therefore treat the stored/validated `redirect_uri` as the only admissible value and never grow a caller-supplied parameter.

**The error plumbing already exists and is dead.** `ParseOauth2RedirectUrlError` (`oxidauth-kernel/src/auth/oauth2/redirect.rs:31-44`, message "unable to create redirect_url: …") is defined but constructed nowhere — a validation hook planned for exactly this step and never wired.

**Verdict.** Not the register's literal "missing params" bug (the bake-in convention is documented and the pinned comment at `:251-254` openly states it), but a real P2: an unenforced invariant with duplicated truth. The server can emit RFC-non-conforming authorization requests (no `client_id`) and self-inflicted `redirect_uri` mismatches without any signal; the two failure modes are invisible in logs (`info!` success path `redirect.rs` handler `:21-24` returns the poisoned URL as a success).

## Impact

- **Operators** configuring/patching oauth2 authorities get zero feedback; misconfiguration (typo'd scope, forgot to re-bake after rotating client_key via OXA-000003) ships as a working authority.
- **End users** of a drifted authority either die on a Google/Microsoft error page with no path back (missing `client_id`/`redirect_uri`), or authenticate fully and then silently fail token exchange (`redirect_uri` drift) — the worst kind of SSO outage: intermittent-looking, provider-side, no local error.
- **Microsoft authorities** additionally have a guaranteed field-vs-URL split: `scopes` is sent at the token endpoint but can only enter the authorization request via the baked URL — two manual copies of one value.
- **No authentication bypass**: the callback still requires a real, single-use IdP code exchanged with `oauth2_secret` (blast radius bounded the same way OXA-000010 argues). Severity stays P2, driven by silent-failure and integrity, not bypass.

## Proposed resolution

**Step 1 — compose identity params authoritatively in `redirect.rs:64-96`.** Keep `redirect_url` as the *endpoint + provider extras* carrier, but strip and re-author the identity trio from the typed fields before appending flavor params:

1. Parse the stored URL; remove any existing `client_id`, `scope`, `redirect_uri` pairs from the query.
2. `append_pair` from fields: `client_id = oauth2_id`, `scope = scopes`, `redirect_uri = redirect_uri.as_str()` (the `url` crate's `query_pairs_mut` percent-encodes correctly — this also fixes the raw-space scope list in the `OAUTH.md:88` example, which `Url::parse` currently normalizes only by luck).
3. Keep the flavor append block as-is (unknown extras like a hand-baked `prompt=consent` survive; identity pairs can no longer duplicate).

This makes `AuthorityParams` the single source of truth, aligning the URL with what both `exchange_token.rs` files already send per RFC §4.1.3.

**Step 2 — composition-time callback consistency check.** Before serving, verify the resolved `redirect_uri`'s last path segment equals `authority.client_key` and its path matches the callback route shape (`/…/oauth2/callback/{client_key}`). Mismatch → return `ParseOauth2RedirectUrlError::Unknown` (now finally wired) with both values quoted; the handler's existing error arm (`api .../redirect.rs:30-37`) answers 400, making the misconfiguration visible on the *first redirect attempt* instead of at callback. Do not compare against an absolute base URL — the server doesn't know its public origin; suffix equality on the key is the load-bearing part and is exactly what OXA-000003 regeneration breaks.

**Step 3 — docs.** Rewrite `docs/OAUTH.md:88` example to a bare `redirect_url` endpoint (`https://accounts.google.com/o/oauth2/v2/auth`) with a note that identity params are composed from the fields; fix the stale step-8 route text (`/auth/oauth2/authenticate/:authority_client_key` → `/callback/{client_key}`).

**Compat.** Existing authorities keep working: their baked identity pairs are replaced by canonical field values on every request — a silent correction, not a break, *except* for anyone who intentionally baked a *different* client identity than `oauth2_id`/`redirect_uri` (that configuration was already guaranteed-broken at the token endpoint per §4.1.3, so no legitimate deployment exists). Changelog note only; no wire-format or schema change. Step 2 will newly 400 for authorities whose baked callback references a stale client_key — that is the bug working as intended; pair the release note with OXA-000003's fix so key regeneration stops producing such states.

**Boundaries (do not duplicate).**
- `state`: minting/verification scheme belongs to **OXA-000010** (nonce store replacing `hash_client_id`); this ticket only touches the lines *around* the `append_pair("state", …)` calls — sequence this change after OXA-000010 lands or rebase; whatever value OXA-000010 produces, it stays appended as `state` here.
- Token-exchange relocation into the strategy and authenticator verification: **OXA-000004**; this ticket changes nothing in `exchange_token.rs` — it only guarantees the authorization URL matches what those senders already emit.
- Callback response statuses and the *other* `gen_redirect_url` (post-login client redirect): **OXA-000037**.
- `client_key` regeneration on PUT: **OXA-000003**; params overwrite semantics on update: **OXA-000016**. Step 2 is the detector for the drift those tickets' fixes prevent.

**Pinned-test flips (`redirect.rs` test module):**
- `BUG(pinned)` marker `:251-254` and the exact-vector assertion `:255-269` flip: under Step 1 the composed Google URL's `scope` pair becomes `"openid email profile"` (the fixture's `scopes` field, `mod.rs:99`) instead of the fixture's baked `"openid email"` (`mod.rs:104`); `client_id`/`redirect_uri` keep the same values but now demonstrably *from the fields*. Rewrite the assertion as: identity pairs equal the `AuthorityParams` fields, flavor pairs present exactly once, baked-but-conflicting pairs *replaced* not duplicated. Delete the marker.
- Microsoft tests (`:283-316`, `:319-335`) and `malformed_authority_params_error` (`:363-379`) pass unchanged — their `contains()`-style assertions hold under composition.
- Fixture `mod.rs:92-110`: align the baked `scope` (or drop baked identity pairs entirely, which is the cleaner post-fix shape).

## Verification

- Module tests: `cargo test -p oxidauth-services --lib auth::strategies::oauth2::redirect` (Google/Microsoft/unknown-key/malformed-params + new cases below).
- New regression tests in the redirect.rs module (drift was the defect — test it directly):
  1. **Adversarial baked URL** — store `https://authorize.example.com/oauth?client_id=WRONG&scope=bogus&redirect_uri=https%3A%2F%2Fevil.example.com%2Fcb&response_type=token`; assert emitted pairs equal `oauth2_id`/`scopes`/`redirect_uri` fields, exactly one `response_type` pair and it is `code`, and no `WRONG`/`evil.example.com`/`token` pair survives.
  2. **Callback/key drift** — store `redirect_uri` whose last segment is a different uuid than the authority's `client_key`; assert the typed "unable to create redirect_url" error naming both keys.
  3. **No caller control** — assert `Oauth2RedirectParams` still carries no URL-bearing field (guards against the open-redirect regression path from §3.1.2.3/§10.15).
- Scoped compile/test across touched crates only: `cargo test -p oxidauth-kernel --lib auth::oauth2::redirect` if the error wiring is extended, and `cargo test -p oxidauth-api oauth2::redirect` for the 400-on-drift handler behavior (throwaway handler-level test acceptable per repo convention).
- Post-fix grep gates: `BUG(pinned)` gone from `redirect.rs` (`grep -rn "BUG(pinned)" src/oxidauth/oxidauth-services/src/auth/strategies/oauth2/redirect.rs` → empty); `BUGS_AND_NOTES.md:72` marked resolved with a note that `:251` was the test marker, defect at `:64-96`.
