INSERT INTO permissions
(realm, resource, action)
VALUES ($1, $2, $3)
RETURNING *
