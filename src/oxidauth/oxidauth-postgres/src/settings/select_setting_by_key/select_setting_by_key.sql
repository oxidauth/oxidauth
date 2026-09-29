SELECT *
FROM settings
WHERE key = $1
LIMIT 1
