SELECT *
FROM user_authorities
WHERE authority_id = $1
AND user_identifier = $2
