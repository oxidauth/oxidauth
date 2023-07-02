DELETE FROM authorities 
WHERE user_id = $1 
RETURNING *
