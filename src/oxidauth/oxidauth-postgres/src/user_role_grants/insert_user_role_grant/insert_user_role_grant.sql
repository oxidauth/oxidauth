INSERT INTO user_role_grants
(user_id, role_id, created_at, updated_at)
VALUES ($1, $2, NOW(), NOW())
RETURNING *