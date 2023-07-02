DELETE FROM refresh_tokens
WHERE expires < $1
RETURNING *
