INSERT INTO settings
(key, value, created_at, updated_at)
VALUES($1, $2, NOW(), NOW())
ON CONFLICT (key)
DO UPDATE SET value = $2, updated_at = NOW()
RETURNING *
