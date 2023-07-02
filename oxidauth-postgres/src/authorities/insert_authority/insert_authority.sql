INSERT INTO authorities
(id, name, client_key, status, strategy, settings, params, created_at, updated_at)
VALUES (COALESCE($1, uuid_generate_v4()), $2, $3, $4, $5, $6, NOW(), NOW())
RETURNING *