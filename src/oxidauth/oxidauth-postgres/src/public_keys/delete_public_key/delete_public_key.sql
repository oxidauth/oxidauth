DELETE FROM public_keys
WHERE id = $1
RETURNING *
