DELETE FROM permissions 
WHERE $1 
RETURNING *