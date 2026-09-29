DELETE FROM permissions
WHERE realm = $1
AND resource = $2
AND action = $3
RETURNING *
