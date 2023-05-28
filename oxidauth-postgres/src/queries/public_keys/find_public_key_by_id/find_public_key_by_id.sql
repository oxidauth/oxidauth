SELECT id, public_key, created_at, updated_at
FROM public_keys
WHERE id = $1
