UPDATE authorities
SET 
    realm = $2,
    resource = $3,
    action = $4,
    updated_at = NOW()
WHERE id = $1
RETURNING *