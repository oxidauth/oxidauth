UPDATE users
SET
    email = $2,
    first_name = $3,
    last_name = $4,
    status = $5,
    profile = $6
WHERE id = $1
RETURNING *
