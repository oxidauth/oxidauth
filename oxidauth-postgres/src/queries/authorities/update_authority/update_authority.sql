UPDATE authorities
SET
    name = $2,
    client_key = $3,
    status = $4,
    strategy = $5,
    params = $6,
    settings = $7
WHERE id = $1
RETURNING *
