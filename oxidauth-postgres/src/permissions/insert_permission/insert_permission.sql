INSERT INTO permissions
(id, realm, resource, action, created_at, updated_at)
VALUES (COALESCE($1, uuid_generate_v4()), $2, $3, $4, NOW(), NOW())
RETURNING *
