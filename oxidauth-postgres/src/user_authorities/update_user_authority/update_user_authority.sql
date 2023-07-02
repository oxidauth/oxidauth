UPDATE user_authorities
SET 
  authority_id = $2,
  user_identifier = $3,
  params = $4,
WHERE user_id = $1
RETURNING *