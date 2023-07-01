INSERT role_role_grant
(parent_id, child_id)
VALUES ($1, $2)
RETURNING *