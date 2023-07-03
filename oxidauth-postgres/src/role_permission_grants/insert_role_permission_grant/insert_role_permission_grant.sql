INSERT INTO role_permission_grants
(id, role_id, permission_id, created_at, updated_at)
VALUES (COALESCE($1, generate_uuid_v4()), $2, $3, NOW(), NOW())
RETURNING *