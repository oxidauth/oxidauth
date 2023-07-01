INSERT INTO authorities
(name, client_key, status, strategy, settings, params, created_at, updated_at)
VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
RETURNING *