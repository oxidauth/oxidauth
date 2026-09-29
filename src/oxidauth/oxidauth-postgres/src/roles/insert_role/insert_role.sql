INSERT INTO roles
(id, name, created_at, updated_at)
VALUES(COALESCE($1, uuid_generate_v4()), $2, NOW(), NOW())
RETURNING *
