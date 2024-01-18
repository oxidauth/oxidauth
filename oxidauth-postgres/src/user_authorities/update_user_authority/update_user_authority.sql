UPDATE user_authorities
SET params = $3
WHERE user_id = $1
AND authority_id = $2
RETURNING *
