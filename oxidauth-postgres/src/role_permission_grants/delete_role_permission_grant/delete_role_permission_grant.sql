DELETE FROM role_permission_grants
WHERE role_id = $1
AND permission_id = $2
RETURNING *
