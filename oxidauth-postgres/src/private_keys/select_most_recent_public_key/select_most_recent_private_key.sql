SELECT
    id,
    private_key,
    created_at,
    updated_at
FROM public_keys
ORDER BY created_at DESC
LIMIT 1
