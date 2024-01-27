INSERT INTO invitations
(COALESCE($1, uuid_generate_v4()) user_id, expires_at, created_at, updated_at)
VALUES ($1, $2, $3, NOW(), NOW())
RETURNING *
