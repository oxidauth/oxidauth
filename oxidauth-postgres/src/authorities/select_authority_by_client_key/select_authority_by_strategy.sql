SELECT *
FROM authorities
WHERE client_key = $1
LIMIT 1
