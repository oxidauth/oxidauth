- [OXA-000073](../docs/tickets/OXA-000073-google-verified-email-enforcement.md) - enforce Google's `verified_email` flag
    - the Google profile path deserialized `verified_email` — making it
      mandatory on the wire — and then discarded it: an address Google
      itself flags unverified passed the boundary untouched, got a
      `user_authorities` row and a full session, making registration an
      effectively anonymous action for exactly the accounts Google warns
      about
    - `GoogleProfile::into_oauth_profile()` now refuses
      `verified_email: false` with the typed `GoogleProfileError::
      EmailUnverified` (sanitized display, `OAuth2StateError` /
      `AppleAuthError` family); `retrieve_google_profile` maps through it,
      so the refusal lands BEFORE any authenticate/register hop — no
      lookup, no registration, no session
    - the parse boundary (`TryFrom<JsonValue>`) stays wire-shape-only and
      the presence pins are unchanged; the "never checked — unverified
      addresses pass this boundary" pin is flipped to the boundary
      refusing `false`
    - end-to-end regression: a fake IdP serving `"verified_email": false`
      produces the typed error with `find_user_authority` and
      `insert_user` never called
    - consistency: Apple already rejects `email_verified: false` at its
      own boundary since OXA-000072; Microsoft is out of scope (Graph
      carries no verified flag — identifier robustness stays OXA-000044)
    - breaking by design for the small population of unverified Google
      accounts: their logins now fail with the typed error; verified
      logins and wire shapes are untouched
