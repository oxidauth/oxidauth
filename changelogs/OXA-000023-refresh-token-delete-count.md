- [OXA-000023](https://www.pivotaltracker.com/story/show/OXA-000023) - bulk refresh-token revocation reports every deleted row; zero tokens is success, not an error
    - `delete_refresh_token_by_user_id` deletes every row of the user
      (`DELETE … WHERE user_id = $1 RETURNING *`) but was executed with
      `fetch_one`, so the count of revoked sessions was destroyed (rows 2..N
      silently dropped) and a zero-token user hard-errored
      `sqlx::Error::RowNotFound`. `fetch_one` → `fetch_all` and the response
      type `RefreshToken` → `Vec<RefreshToken>` across the whole contract
      chain: `oxidauth-repository::refresh_tokens::delete_refresh_token_by_user_id::DeleteRefreshTokenByUserIdQuery`
      (trait method now `Result<Vec<RefreshToken>, BoxedError>`), the
      `PgRefreshTokenRepository` impl, and the `forgot_password` test mock.
      Zero matches is now a success with an empty `Vec`, never an error; the
      impl carries a doc-comment pinning that contract and logs
      `tracing::info!(count = deleted.len())` so bulk revocations are
      countable in server logs. No SQL change, no wire/DTO/migration change.
    - **One consumer-visible behavior delta:**
      `POST /api/v1/auth/username_password/forgot_password` for a user with a
      valid TOTP secret and **zero** live refresh tokens now returns
      **200 + code** instead of **400 `"database row not found"`**. Integrators
      that (accidentally) treated the 400 as "user has no sessions" must
      repoint; the DELETE scope itself was always unchanged.
    - **BREAKING (internal API):** dead kernel alias
      `oxidauth_kernel::refresh_tokens::delete_refresh_token_by_user_id::DeleteRefreshTokenByUserIdService`
      deleted (its `...ServiceTrait` twin was already removed with OXA-000069's
      kernel Service cutover) — zero
      implementors/consumers workspace-wide (`grep -rn
      "DeleteRefreshTokenByUserIdService" src/` → 0). The
      `DeleteRefreshTokenByUserId` request DTO and the module's re-exports
      stay. Out-of-tree implementers restore via one revert.
    - Both pinned pg tests flipped: `it_should_delete_only_the_target_users_tokens`
      now asserts the response reports **both** revoked rows (`len() == 2`,
      all rows belong to the target user, deleted id-set equality compared
      sorted — the `RETURNING` has no `ORDER BY`), `BUG(pinned)` marker
      deleted; `it_should_error_when_the_user_has_no_tokens` renamed to
      `it_should_return_an_empty_vec_when_the_user_has_no_tokens` asserting
      `Ok` + `is_empty()`. All DB-state scoping assertions (other user's
      token survives) unchanged. The `forgot_password` use-case tests assert
      only call-log contents and stay green with the mock's type change.
