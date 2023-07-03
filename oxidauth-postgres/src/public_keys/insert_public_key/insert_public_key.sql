INSERT INTO public_keys
(id, private_key, public_key, created_at, updated_at)
VALUES (COALESCE($1, generate_uuid_v4()), $2, $3, NOW(), NOW())
