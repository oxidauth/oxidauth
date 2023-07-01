DELETE FROM user_role_grants
WHERE user_id = $1
AND WHERE role_id = $2
RETURNING *