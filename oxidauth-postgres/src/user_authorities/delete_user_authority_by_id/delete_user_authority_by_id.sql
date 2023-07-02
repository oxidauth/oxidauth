DELETE FROM user_authorities
WHERE id = $1
RETURNING *