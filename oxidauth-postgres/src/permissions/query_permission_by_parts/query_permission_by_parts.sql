SELECT * 
FROM permissions
WHERE realm = $1
OR WHERE resource = $2
OR WHERE action = $3