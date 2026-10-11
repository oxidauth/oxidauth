- [OXA-000004](../docs/tickets/OXA-000004-oauth2-authenticator-no-verification.md) step 1 - close the generic-entry oauth2 auth bypass
    - `POST /v1/auth/authenticate` (and `/v1/auth/register`) could mint a full
      session for an oauth2-strategy authority from `params.email` alone,
      because the oauth2 `Authenticator` verifies nothing — its real
      verification (state + IdP code exchange) lives in the callback flow
    - adds `AuthEntry` (`External`/`Internal`) to `AuthenticateParams` and
      `RegisterParams`, `serde(skip)`ped on purpose: deserialized wire input
      can only ever produce `External`, so a request body cannot claim the
      trusted entry
    - `AuthenticateUseCase` and `RegisterUseCase` now reject `External`
      requests against any non-`username_password` strategy with the typed
      `StrategyEntryNotAllowedError` before identifier resolution or any
      writes; the oauth2 message points callers at
      `GET /v1/auth/oauth2/callback/{client_key}`
    - in-process hops that legitimately drive other strategies set
      `AuthEntry::Internal`: `AuthenticateOrRegisterUseCase` (authenticate +
      register fallback), the kernel `From<&AuthenticateOrRegisterParams>`
      conversions, and first-boot admin provisioning in `bootstrap`
    - `build_authenticator`/`build_registrar` return an error for the
      unimplemented `single_use_token` strategy instead of `unimplemented!()`
      panicking; the two `should_panic` pins flip to error assertions
    - no wire-shape change: `entry` is never serialized, and the
      `username_password` login path is untouched
    - deploy note: any caller exploiting the bypass (direct generic
      authenticate against an oauth2 authority) starts failing with a 4xx —
      by design
