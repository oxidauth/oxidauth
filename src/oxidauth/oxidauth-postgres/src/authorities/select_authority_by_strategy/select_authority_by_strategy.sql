SELECT *
FROM authorities
WHERE strategy = $1
ORDER BY created_at, id
LIMIT 1
