DELETE FROM user_permission_grants
WHERE user_id = $1
AND permission_id = $2
RETURNING *
