- [OXA-000010](../docs/tickets/OXA-000010-aor-state-replay.md) - single-use oauth2 state nonces
    - the oauth2 `state` was `Argon2(client_key bytes)` — a hash of PUBLIC
      data: anyone knowing an authority's public `client_key` could mint
      unlimited valid states offline, every state ever issued stayed valid
      forever (login-CSRF / session-fixation enabler), and verification ran
      attacker-chosen KDF cost parameters on an unauthenticated GET (DoS
      amplifier)
    - mints are now opaque 128-bit CSPRNG nonces (base64url) recorded
      server-side in a new `oauth_states` table as `sha256(state)` with a
      10-minute TTL; expired rows are garbage-collected lazily on every mint
    - the callback consumes the state atomically BEFORE any IdP contact:
      `DELETE ... RETURNING` keyed by `(state_hash, client_key, live
      expires_at)` — single-use under concurrency, bound to its authority,
      and dead the moment it is spent
    - adds the typed `OAuth2StateError { Malformed, Invalid }`; unknown,
      expired, and spent nonces deliberately collapse into `Invalid` so the
      endpoint leaks no oracle over the nonce space; overlong/empty states
      are refused on shape before any hashing work
    - new entity per the README recipe: migration
      `20261011224416_create_oauth_states.sql`, `InsertOAuthStateQuery` /
      `ConsumeOAuthStateQuery` in `oxidauth-repository`,
      `PgOAuthStateRepository` in `oxidauth-postgres` (incl. concurrent
      single-use and digest-at-rest round-trip tests), wired in
      `oxidauth-api/src/provider/services.rs`
    - argon2/`SaltString` imports removed from `redirect.rs` and
      `authenticate_or_register.rs`; the `hash_client_id` function and the
      "state must be an argon2 hash of the client key" pins are deleted —
      replaced by nonce-unique / bound-row / replay / wrong-authority /
      malformed / store-failure tests
    - wire shapes are unchanged (`state` was already an opaque `String`
      everywhere; the SPA only replays it) — no hurl, SDK, or frontend
      changes
    - deploy note: argon2-format states minted pre-deploy FAIL verification
      by design (rejecting every previously minted state is the point of the
      fix); users mid-flow retry the login once. Do not dual-read the old
      format
