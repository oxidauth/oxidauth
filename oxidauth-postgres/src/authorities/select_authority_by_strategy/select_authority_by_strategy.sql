SELECT *
FROM authorities
WHERE strategy = $1
LIMIT 1
