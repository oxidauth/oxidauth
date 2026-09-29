INSERT INTO user_permission_grants
(user_id, permission_id, created_at, updated_at)
VALUES ($1, $2, NOW(), NOW())
RETURNING *