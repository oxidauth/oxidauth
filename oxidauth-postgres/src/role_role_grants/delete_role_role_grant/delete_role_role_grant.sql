DELETE FROM role_role_grants
WHERE parent_id = $1
AND child_id = $2
RETURNING *
