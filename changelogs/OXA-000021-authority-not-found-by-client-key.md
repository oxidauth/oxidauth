- [OXA-000021](https://www.pivotaltracker.com/story/show/OXA-000021) - unknown `client_key` reports the domain error, not raw `RowNotFound`
    - `select_authority_by_client_key` declared `Response = Option<Authority>` but
      queried with `fetch_one`, so a missing key surfaced as
      `sqlx::Error::RowNotFound` and the `None` branch was unreachable dead API —
      every `ok_or_else(AuthorityNotFoundError::client_key)` arm downstream never
      fired against the real repository. `fetch_one` → `fetch_optional` +
      `.map(TryInto::try_into).transpose()?` (the working `select_authority_by_strategy`
      sibling's shape); real DB failures still arrive as `Err`, which is exactly the
      distinction the `Option` was bought for. No SQL/trait/DTO/wire change.
    - **User-visible body wording changes on five 400 endpoints** —
      `POST /auth/authenticate`, `POST /auth/register`, `GET|POST /auth/oauth2/redirect`,
      `POST /totp/validate`, `POST /users/{user_id}/authorities`: status codes do not
      move (the 400-not-404 convention holds), but the error envelope now carries
      `display: "authority not found by client_key: <uuid>"` /
      `debug: "ClientKey(<uuid>)"` instead of the sqlx driver text
      (`"database row not found"` / `"RowNotFound"`). Consumers matching
      `"RowNotFound"` on these routes must repoint at the domain wording; the
      in-tree suite is the single `hurl/tests/authenticate.hurl` assertion, flipped
      to `jsonpath "$.errors[0].display" contains "authority not found by client_key"`.
      Other `RowNotFound` hurl assertions (users/roles/exchange/authorities/
      public_keys) target different `fetch_one` queries and are untouched.
    - Byte-identical for users: the OAuth2 callback page (service error swallowed
      into the generic browser text) and `POST /auth/username_password/update_password`
      (`{ "success": false }` envelope, its deliberate Err+None flatten kept);
      found-key paths unchanged. The authenticate-or-register arm now constructs
      `AuthorityNotFoundError::client_key` instead of a bare string (message still
      only logged by the callback handler).
    - Test-side, no contract change: the pinned pg test
      `it_should_error_not_return_none_for_missing_client_key` renamed to
      `it_should_return_none_for_missing_client_key` asserting `is_none()`
      (`BUG(pinned)` marker deleted); the one AOR unit-test string assertion
      follows the unified error text.
