INSERT INTO authorities (
    name, client_key,
    status, strategy,
    params, settings
) VALUES ($1, $2, $3, $4, $5, $6)
RETURNING *
