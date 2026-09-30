-- OXA-000013: invitations.user_id was only ever indexed (invitations_user_id),
-- never a FK, so the repository layer accepted invitations for deleted/unknown
-- users. Clean up the dangling rows the missing constraint let in first — an
-- invitation for a nonexistent user cannot be repaired, only discarded. NOTE:
-- sqlx-cli discards SELECT results during `migrate run`, so the count below is
-- inert there (it prints only when applied via psql). Operators wanting the
-- count BEFORE upgrading: run the same NOT EXISTS predicate as a SELECT first
-- (see changelog OXA-000013).
SELECT COUNT(*) AS dangling_invitations FROM invitations i
WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = i.user_id);

DELETE FROM invitations i
WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = i.user_id);

-- Deliberate deviation from the house default: every sibling user-referencing
-- table (user_authorities, user_role_grants, refresh_tokens, totp_secrets) uses
-- ON DELETE CASCADE, but the invitation row is the outstanding claim token for
-- the placeholder user, so CASCADE would let DELETE /users/{id}
-- (oxidauth:users:delete) silently revoke pending offers — an entitlement
-- cross-leak against oxidauth:invitations:delete. RESTRICT forces the correct
-- two-step cleanup: revoke the invitation (DELETE /v1/invitations/{id}), then
-- delete the user. Accepted users keep no invitations row, so no real account
-- is ever blocked. The existing invitations_user_id index already serves the
-- FK's child-row check on user delete; no new index needed.
ALTER TABLE invitations
ADD CONSTRAINT invitations_users_fk
FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE RESTRICT;
