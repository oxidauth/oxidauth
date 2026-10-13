# OXA-000072 — Sign in with Apple: third OAuth2 flavor (`Apple`), with server-signed ES256 client secrets and id_token-based profiles

**Type:** feature · **Status:** open
**Milestone:** 3 of the Apple-OAuth series (M1 OXA-000004 step 1, M2 OXA-000010, M4 = Google `verified_email` follow-up)

## Goal

Add `Apple` as an `OAuthFlavors` variant so an authority row can offer Sign in
with Apple through the existing `POST /oauth2/redirect` → IdP → callback →
`AuthenticateOrRegister` pipeline, without inventing a second login path.

## Why Apple is not just another GET-profile flavor

1. **No static client secret.** Apple's token endpoint authenticates with a
   *client secret JWT* the integrator signs per request: ES256, `iss` = Team
   ID, `aud` = `https://appleid.apple.com`, `sub` = Services ID, `exp` ≤ 6
   months (we use 5 minutes), header `kid` = the key's Key ID. There is no
   `oauth2_secret` to store; the secret is derived from an Apple-issued `.p8`
   private key.
2. **Identity arrives in the token response, not a profile endpoint.** Apple
   has no userinfo GET. The `id_token` JWT (RS256, JWKS-hosted keys) carries
   `email`/`sub`, and it must be verified: signature against
   `https://appleid.apple.com/auth/keys`, `iss`, `aud` = Services ID, and
   **`nonce` = base64url(SHA-256(state))** — Apple hashes the nonce we send at
   authorize time, so the OXA-000010 state nonce finally does its *actual*
   OIDC job: binding the token to a consumed, single-use, server-minted nonce.
3. **The callback comes back as a form POST.** `response_mode=form_post` —
   Apple `POST`s `code`/`state`/`id_token` to the redirect URI instead of
   appending query params (the GET handler stays for Google/Microsoft).

## Key material policy (decision)

Private keys are **never** authority-JSONB material (same reasoning as the
username_password `pepper_env_var_key` indirection: DB rows are readable by
anyone with DB read + the `params` column routinely appears in API responses).
The authority row carries three new optional params:

- `apple_team_id` — Apple Team ID (`iss`)
- `apple_key_id` — the Key ID of the Services key (`kid` header)
- `apple_private_key_env_var` — **name** of the environment variable holding
  the PKCS#8 PEM private key

The row names an env var; only the process environment holds the key bytes.
A missing/misconfigured env var fails the exchange with a typed error, not a
panic.

## Scope

- `OAuthFlavors::Apple` + additive `Option` fields on `AuthorityParams`
  (existing Google/Microsoft rows deserialize unchanged).
- `oauth2/apple/` module: `client_secret.rs` (ES256 signing via
  `jsonwebtoken`, already vendored for the kernel's JWT),
  `exchange_token.rs` (form POST to `exchange_url`, typed+redacted errors —
  no `map_err(err.to_string())` raw-text propagation, no `unimplemented!()`),
  `id_token.rs` (JWKS fetch, kid lookup, RS256 verify, `iss`/`aud`/`exp`
  validation, nonce == SHA-256(state), email extraction).
- Display names are **not** requested: Apple only sends the `user` JSON once
  (first consent) and it can't be re-fetched; storing it would depend on one
  irreversible moment. Registration proceeds with email only.
- Redirect arm: `response_type=code&response_mode=form_post&state=<nonce>`
  (identity pairs stay baked into the stored `redirect_url`, matching the
  OXA-000043-pinned Google/Microsoft contract).
- Callback: `POST /v1/auth/oauth2/callback/{client_key}` form handler mapping
  to the same `AuthenticateOrRegisterParams`; an Apple `error` field (user
  denial) is logged and maps to the static error text; the form's `id_token`
  is **ignored** — the server exchanges the `code` itself and verifies the
  `id_token` returned by the *token endpoint*, never trusting whatever was
  POSTed.
- Docs: README + example.env setup section.

## Deliberately NOT in scope

- Per-browser-session state binding (unchanged from OXA-000010 design).
- Google/MS id_token verification (M4 / OXA-000037 territory; Apple does it
  right from day one).
- `emailIsVerified`-style enforcement beyond rejecting `email_verified:
  false` (Apple-relay private emails are legitimate).
- OXA-000037 (callback 200-on-error) — the POST handler matches the pinned
  GET behavior until that ticket lands.

## Verification (tests that ship with this ticket)

- `client_secret`: signed JWT decodes with the paired public key; claims
  `iss`/`aud`/`sub`/`exp≈now+5m`; `kid` header present; missing params /
  missing env var / garbage PEM each produce typed errors (never a panic).
- `id_token`: fixture-signed RS256 token verifies against a fixture JWKS and
  yields the email; wrong `kid`, wrong `aud`, expired token, nonce mismatch
  (including a token bound to a *different* state) and `email_verified:
  false` are all rejected.
- `redirect`: Apple URL appends `response_type=code`,
  `response_mode=form_post`, fresh single-use state (same OXA-000010 store
  assertions as the other flavors).
- `callback` form: POST with code+state reaches the service with identical
  params to the GET path; missing code → 400; Apple `error` field → static
  error text; form `id_token` never reaches the service params.
- Exchange wire contract: form-body field names pinned by test
  (`client_id` = Services ID, `client_secret` = signed JWT,
  `grant_type=authorization_code`, `code`, `redirect_uri`).

## Impact

New capability; zero behavior change for existing authorities (additive
serde fields, additive enum variant, additive route method). Deploy: existing
DB schema already accepts `provider LIKE 'oauth2%'`, so no migration; ops
adds one env var per Apple authority (the `.p8` PEM) plus the two string
params on the authority row.
