INSERT INTO roles
(id, name)
VALUES($1, $2)
RETURNING *