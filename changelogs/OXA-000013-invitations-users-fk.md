- [OXA-000013](https://www.pivotaltracker.com/story/show/OXA-000013) - `invitations.user_id` gains the missing FK `invitations_users_fk ... ON DELETE RESTRICT` (register SEC-13)
    - The column was `NOT NULL` and indexed but never constrained, so the
      repository layer persisted invitations for deleted/unknown users;
      accepting such a row consumed the invitation, then failed on
      `user_authorities_users_fk`, surfacing an unrecoverable 400.
    - **Storage migration:** new
      `20260930180000_add_invitations_users_fk.sql` first DELETEs dangling
      invitations (they were already unusable — accepting one only consumed
      it and failed), then adds
      `invitations_users_fk FOREIGN KEY (user_id) REFERENCES users(id)`.
      sqlx-cli hides migration SELECT output — use the pre-check query below
      to get the count before upgrading.
      The existing `invitations_user_id` index is retained as-is; it serves
      the FK's child-row check.
    - **RESTRICT, deliberately not CASCADE** (the siblings' default): the
      invitation row is the outstanding claim token for the placeholder
      user, so CASCADE would let `DELETE /users/{id}` silently revoke
      pending offers — an entitlement cross-leak against
      `oxidauth:invitations:delete`. CASCADE remains the drop-in alternative
      if product rejects the friction; not a silent fallback.
    - **BREAKING for user deletion:** `DELETE /v1/users/{user_id}` for a
      user holding a *pending* invitation now returns 400 (SQLSTATE 23503,
      `invitations_users_fk` in the error payload). Runbook: revoke the
      invitation first (`DELETE /v1/invitations/{id}`,
      `oxidauth:invitations:delete`), then delete the user. Accepted users
      keep no `invitations` row, so no real account is ever blocked.
    - **Ops:** the cleanup DELETE discards pre-existing dangling rows on
      upgrade (pre-check: `SELECT COUNT(*) FROM invitations i WHERE NOT
      EXISTS (SELECT 1 FROM users u WHERE u.id = i.user_id)` — zero on the
      dev DB at fix time). No API/DTO/Rust-surface change; violations
      surface through the existing `BoxedError` path like every other FK.
    - Repo artifacts flipped in the same commit: `insert_invitation` pin
      `it_should_accept_an_unknown_user_because_the_column_has_no_fk` →
      `it_should_reject_an_unknown_user_because_of_the_users_fk` (23503 +
      zero-row assert, `BUG(pinned)` marker deleted), plus a new
      RESTRICT-on-user-delete test pinning the delete behavior. The
      `accept_invitation.rs` SEC-2 pin and `oxidauth-rs` CLI-6 markers are
      untouched.
