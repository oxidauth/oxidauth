INSERT INTO role_permission_grants
(role_id, permission_id, created_at, updated_at)
VALUES ($1, $2, NOW(), NOW())
RETURNING *
