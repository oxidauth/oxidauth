UPDATE authorities
SET 
    name = $2,
    client_key = $3,
    status = $4,
    strategy = $5,
    settings = $6,
    params = $7,
    updated_at = NOW()
WHERE id = $1
RETURNING *