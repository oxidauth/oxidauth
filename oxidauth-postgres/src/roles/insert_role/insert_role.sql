INSERT INTO roles
(id, name, created_at, updated_at)
VALUES(COALESCE($1, generate_uuid_v4()), $2, NOW(), NOW())
RETURNING *