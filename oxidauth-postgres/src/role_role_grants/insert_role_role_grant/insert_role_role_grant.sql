INSERT role_role_grant
(parent_id, child_id, created_at, updated_at)
VALUES ($1, $2, NOW(), NOW())
RETURNING *