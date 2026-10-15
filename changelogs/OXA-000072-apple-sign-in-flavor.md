- [OXA-000072](../docs/tickets/OXA-000072-apple-sign-in-flavor.md) - Sign in with Apple OAuth2 flavor
    - new `OAuthFlavors::Apple`: authorities with `"flavor": "Apple"` run the
      existing redirect → IdP → callback → authenticate-or-register pipeline
      with Apple's three deviations handled inside a new `oauth2/apple/`
      module — per-request ES256 client-secret JWTs, id_token-based identity,
      and a form-POST callback
    - key material policy: the authority row carries `apple_team_id`,
      `apple_key_id`, and `apple_private_key_env_var` (the *name* of the env
      var holding the PKCS#8 PEM) — private key bytes never enter the
      database, mirroring the username_password pepper indirection; all
      three fields are additive serde optionals, so stored Google/Microsoft
      rows deserialize unchanged and no migration is needed (the
      `provider LIKE 'oauth2%'` CHECK already accepts `oauth2.apple` rows)
    - `build_client_secret` signs the per-request JWT Apple demands instead
      of a static secret: ES256, `iss` = Team ID, `aud` =
      `https://appleid.apple.com`, `sub` = Services ID, fixed 5-minute
      expiry, `kid` header — via `jsonwebtoken`, already pinned by the kernel
    - `exchange_apple_token` POSTs the code and returns the `id_token`;
      failures are typed (`AppleAuthError`) and sanitized — IdP bodies stay
      in redacted logs, the caller gets status + OAuth error code only (no
      `map_err(err.to_string())` prose, no `unimplemented!()` paths)
    - `verify_apple_id_token` is where the state nonce finally does its real
      OIDC job: RS256 signature against Apple's JWKS (kid-selected),
      `iss`/`aud`/`exp` checks, and `nonce == base64url(sha256(state))` with
      the *consumed* single-use state from OXA-000010 — a captured Apple
      token from another login attempt cannot authenticate this one;
      `email_verified: false` and missing emails are rejected; display names
      are deliberately ignored (Apple sends them only on first consent, so
      the flow refuses to depend on them)
    - redirect appends `response_type=code&response_mode=form_post&state=<nonce>`;
      `POST /v1/auth/oauth2/callback/{client_key}` accepts Apple's form
      callback (the GET handler stays for Google/Microsoft), maps Apple's
      `error` field (user denial) to the static error text without touching
      the service, and never forwards the browser-posted `id_token`/`user`
      fields — identity is re-derived from the server-side code exchange
      (axum gains the `form` feature)
    - misconfiguration fails typed, never panics: missing `apple_*` params,
      unset env var, and unusable PEM each produce their own
      `AppleAuthError` at exchange time
    - tests: signed-token round-trips with throwaway EC/RSA fixtures,
      replayed-wrong-nonce / wrong-audience / expired / unknown-kid /
      unverified-email rejections, wire-contract pins, form-POST callback
      tests, and an end-to-end apple login against a fake IdP serving
      `/token` + `/keys`
    - docs: `example.env` var block and `docs/AUTHORITIES.md` setup section
      (Services ID, `.p8` key, params table)
