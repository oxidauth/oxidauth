DELETE FROM user_permission_grants
WHERE user_id = $1
AND WHERE permission_id = $2