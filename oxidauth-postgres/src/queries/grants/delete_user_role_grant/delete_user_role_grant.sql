DELETE FROM user_role_grants
WHERE user_id = $1
AND role_id = $1
RETURNING *
