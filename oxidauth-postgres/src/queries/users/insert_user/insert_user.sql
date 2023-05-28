INSERT INTO users (
    username, email,
    first_name, last_name,
    status, kind, profile
)
VALUES ($1, $2, $3, $4, $5, $6, $7)
RETURNING *
