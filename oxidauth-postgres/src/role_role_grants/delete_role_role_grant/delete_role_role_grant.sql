DELETE FROM role_role_grants
WHERE parent_id = $1
OR WHERE child_id = $2
RETURNING *