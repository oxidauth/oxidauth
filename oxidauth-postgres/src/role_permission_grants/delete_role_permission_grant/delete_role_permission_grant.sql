DELETE FROM role_permissions_grants
WHERE role_id = $1
OR WHERE permission_id = $2
returning *