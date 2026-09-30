- [OXA-000009](https://www.pivotaltracker.com/story/show/OXA-000009) - `users.status` becomes enforceable: four credential gates refuse `disabled` accounts (login, refresh, 2FA validation, password recovery); the invitation flow stops honouring a client-supplied status
    - **The account-disable control now exists server-side.** `users.status` was
      write-only — stored, indexed (`users_status_idx`), echoed by `/users`, and
      consulted by *nothing*. Every credential path that mints or renews
      authority now gates on the user row, policy **(A) "disable means
      disable"**:
      - `authenticate` — the `SelectUserByIdQuery` fetch moved **out of** the
        `TotpSettings::Enabled` arm, so it runs on every login, and a
        `Disabled` account is refused *before* the signing key, the permission
        tree, and the refresh insert, and before the 2FA webhook (no code is
        ever relayed). The check sits **after** the authenticator call, so a
        wrong password still reports the password error — `disabled` is not an
        existence/status oracle for callers without valid credentials.
      - `exchange_refresh_token` — this path never loaded the user, so a
        disabled session renewed forever; one `SelectUserByIdQuery` read per
        refresh now rejects it after the expired-token hygiene leg. The
        still-valid token row is deliberately **not** deleted or rotated —
        nothing is minted and the row stays auditable. (The ticket's SQL-join
        variant was taken as too invasive for the first cut; the ticket itself
        blesses the explicit dependency.)
      - `totp/validate` — the leg that mints the privileged full-entitlement
        JWT gained the same gate, after the code check (a wrong code still
        answers `"invalid totp code"`) and before any JWT material.
      - `username_password/update_password` (recovery, the item as filed) — a
        fifth `SelectUserByIdQuery` dependency; the gate sits after the
        user-authority lookup and **before** the TOTP secret fetch, so a
        disabled account can neither spend/confirm a 600 s reset code nor
        overwrite the stored password hash. The route envelope is unchanged:
        every failure, including `"account is disabled"`, still answers
        `200 {success:false}` (typed errors: OXA-000042, deferred).
      Each refusal logs `tracing::warn!` with `user_id`, so disabled-account
      attempts are auditable for the first time.
    - **`Invited` decision (Step 0):** only `Disabled` fails closed. `Invited`
      accounts keep authenticating and renewing — unchanged behaviour, now
      pinned by a test. Meaningful `Invited` semantics (and whether
      self-service registration should mint it) remains a separate product
      item; `UserStatus::default()` was NOT touched.
    - **Write-side hole closed (behaviour change):** `status` is removed from
      `oxidauth_kernel::AcceptInvitationUserParams` and the
      `From<(Uuid, &AcceptInvitationUserParams)> for UpdateUser` mapping now
      emits `status: None`, letting `UpdateUserUseCase` backfill the stored
      value. `POST /api/v1/invitations/{invitation_id}` can no longer be used
      to self-assign a status; transitions belong to the admin `update_user`
      route only. Serde stays permissive (no `deny_unknown_fields` in the
      workspace), so legacy clients that still send `"status"` keep
      deserialising — the field is simply no longer honoured. **Callers that
      relied on inviting with a preset status are affected.** The `oxidauth-rs`
      invitation wrapper's canned payload dropped the field; the
      update/forgot-password DTOs are unchanged.
    - **BREAKING on deploy (release note):** run
      `SELECT count(*) FROM users WHERE status = 'disabled'` first — those
      accounts are logging in successfully *today* and will start failing
      (login, refresh, 2FA validation, recovery) the moment this ships. That
      is the fix working; tell whoever disabled them months ago. No schema or
      data migration; no permission changes.
    - **Tests:** new legs — recovery: disabled + valid code → `Err` with an
      empty update log *and* an empty TOTP-secret log (write never attempted,
      code never consulted); login: disabled + correct password → refused with
      no key/tree/refresh ops, gate runs after the credential path, and a
      disabled account never reaches the 2FA relay; refresh: disabled → exact
      op log `[select_token, find_user]` (no rotation, no wipe), `Invited`
      still rotates, plus a user-lookup failure leg; totp/validate: disabled +
      valid code → refused before any JWT material, disabled + wrong code
      still answers `"invalid totp code"` before the user row is consulted.
      Flipped pins: the invitation `UpdateUser` snapshot now asserts
      `status: None` plus a legacy-body-with-`"status"` leg proving it is
      never honoured; the `authenticate_or_register` `MockUserById`
      booby-trap ("user lookup must not be consulted while totp is disabled")
      inverted into an enabled-user answer — the authenticate user load is now
      unconditional by design. `BUG(pinned)` markers survive untouched: the
      masked lookup failure, the swallowed update failure, the comment pin
      (now reading "the only *credential* gate is the TOTP code; the status
      gate lives above it"), and the postgres `it_should_overwrite_params`.
      No hurl coverage exists for either recovery route; the e2e leg named in
      the ticket's Verification rides the integration pass, not this ticket.
