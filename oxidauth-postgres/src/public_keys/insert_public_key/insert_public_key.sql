INSERT INTO public_keys
(private_key, public_key, created_at, updated_at)
VALUES ($1, $2, NOW(), NOW())
