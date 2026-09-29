DELETE FROM authorities
WHERE id = $1
RETURNING *
