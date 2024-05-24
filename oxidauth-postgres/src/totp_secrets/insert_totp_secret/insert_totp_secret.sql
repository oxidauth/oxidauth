INSERT INTO totp_secrets
(id, user_id, totp_secret, created_at)
VALUES (uuid_generate_v4(), $1, $2, NOW())
RETURNING *
