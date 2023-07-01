INSERT INTO refresh_tokens
(user_id, authority_id, expires_at, created_at, updated_at)
VALUES ($1, $2, $3, NOW(), NOW())
