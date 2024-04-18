INSERT INTO auth_keys
(id, user_id, key, created_at, updated_at)
VALUES (uuid_generate_v4(), $1, $2, NOW())
RETURNING *
