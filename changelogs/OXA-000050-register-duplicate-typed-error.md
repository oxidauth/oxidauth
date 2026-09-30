- [OXA-000050](https://www.pivotaltracker.com/story/show/OXA-000050) - duplicate username reports `UserAlreadyExistsError`, not raw Postgres text
    - A duplicate username was never a domain condition: registrars hold no
      repository handle (the `Registrar` contract is params in / DTOs out), so the
      insert died on the `users_username_key` unique index and the raw
      `sqlx::Error::Database` was boxed verbatim. `POST /api/v1/auth/register` —
      unauthenticated by design — answered 400 with SQLSTATE `23505`, the internal
      constraint name and Postgres's own DETAIL line, offending username included.
      Captured live before the fix:
      `"debug": "Database(PgDatabaseError { code: \"23505\", message: \"duplicate key value violates unique constraint \\\"users_username_key\\\"\", detail: Some(\"Key (username)=(<name>) already exists.\"), constraint: Some(\"users_username_key\"), … })"`,
      `"display": "error returned from database: duplicate key value violates unique constraint \"users_username_key\""`,
      plus a `source` repeating it a third time.
    - New kernel error `oxidauth_kernel::users::UserAlreadyExistsError`
      (`UserAlreadyExistsError::username(&Username) -> Box<Self>`, the
      `UserNotFoundError` / `UserAuthorityNotFoundError` shape): `Display` is the
      sanitized wire copy `username is already taken` — no SQLSTATE, no constraint
      name, no username echo, since `Display` reaches the body of a public
      endpoint — while `Debug` keeps the type name and the username for `tracing`.
      It deliberately has **no** `source()`: `into_error` renders the cause's
      `Display` into the envelope, which would have shipped the driver text again.
    - `PgUserRepository::insert_user` maps SQLSTATE `23505` on the
      `users_username_key` constraint to it — the same persistence boundary
      where `RowNotFound` already becomes `UserAuthorityNotFoundError`. A
      `users_pkey` collision (a client-supplied `id` re-POSTed with a fresh
      username) is an id clash, not a taken username, and stays raw (pinned:
      `it_should_propagate_primary_key_collision_raw`).
      Still zero check-then-insert: the unique index
      stays the sole arbiter, so the translation is TOCTOU-safe by construction
      (two concurrent registrations still cannot both win). Every other DB error
      propagates raw and byte-identically (verified: a `VARCHAR(64)` overflow is
      `22001` and stays the raw `sqlx::Error`); the driver text plus the username
      is now logged (`tracing::warn!`) instead of handed to the caller.
    - **Consumer-visible delta: status code unchanged (400), body text changes.**
      `POST /api/v1/auth/register` and `POST /api/v1/users` (both go through
      `insert_user`) now answer
      `{"name":"BoxedError","display":"username is already taken","debug":"UserAlreadyExistsError { username: Username(\"<name>\") }"}`
      with `success:false`, `errors[0]` present and `status_code` still absent —
      the `source` key is gone entirely. Anything substring-matching
      `"duplicate key"`, `users_username_key` or `23505` on those two routes must
      repoint at `UserAlreadyExistsError` (or the `display` copy). The OAuth2
      auto-register callback inherits the typed error but its body stays behind
      OXA-000037's swallowed generic text; `permissions`/`roles`/grant-table
      `23505`s are deliberately untouched and still raw.
    - Test flips (they pinned the leak, so they are contracts now): both hurl
      files move from `$.errors[0].debug contains "duplicate key"` to
      `contains "UserAlreadyExistsError"` plus the sanitized `display` and
      no-`23505`/no-constraint/no-`source` guards
      (`hurl/tests/register.hurl`, `hurl/tests/users.hurl` — each run twice per
      invocation, green); the pg pin `it_should_reject_duplicate_username`
      down-casts the typed error and keeps its SQLSTATE-level check via raw SQL
      (the index really did say no, one row only); new
      `it_should_propagate_non_constraint_errors_raw` proves only `23505` is
      translated; `duplicate_username_propagates_without_writing_a_user_authority`
      now down-casts and still proves no `insert_user_authority`/`private_key`
      write follows the failure.
    - Retired pin, no behavior change: the registrar's `BUG(pinned)`
      `does_not_reject_a_duplicate_username` becomes
      `duplicate_detection_lives_at_the_insert_boundary`, recording the intentional
      layering (uniqueness is arbitrated by the DB at the insert boundary; the
      registrar stays pure). Deliberately out of scope, unchanged: 409-vs-400 and
      the username-existence oracle (OXA-000005), the non-transactional register
      flow, `user_authorities` 23505 wording (noted in-code for its own decision).
