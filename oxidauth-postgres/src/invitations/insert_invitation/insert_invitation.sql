INSERT INTO invitations
(id, user_id, expires_at, created_at, updated_at)
VALUES (COALESCE($1, uuid_generate_v4()), $2, $3, NOW(), NOW())
RETURNING *
