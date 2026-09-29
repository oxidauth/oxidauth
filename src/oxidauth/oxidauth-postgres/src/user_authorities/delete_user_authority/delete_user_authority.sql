DELETE FROM user_authorities
WHERE user_id = $1
AND authority_id = $2
RETURNING *
