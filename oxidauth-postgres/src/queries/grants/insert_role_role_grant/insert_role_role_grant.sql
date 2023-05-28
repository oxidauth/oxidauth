INSERT INTO role_role_grants
(parent_id, child_id)
VALUES ($1, $2)
RETURNING *
