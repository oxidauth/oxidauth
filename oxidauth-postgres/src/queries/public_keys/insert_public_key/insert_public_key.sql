INSERT INTO public_keys
(public_key, private_key)
VALUES ($1, $2)
RETURNING id, public_key, created_at, updated_at
