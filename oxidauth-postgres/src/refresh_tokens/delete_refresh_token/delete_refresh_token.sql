DELETE FROM refresh_tokens
WHERE id = $1
RETURNING *
