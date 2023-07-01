INSERT INTO users
(id, kind, status, username, email, first_name, last_name, profile, created_at, updated_at)
VALUES (COALESCE($1, uuid_generate_v4()), $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
RETURNING *

