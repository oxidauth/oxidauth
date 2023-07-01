UPDATE roles
SET 
    name = $2
WHERE id = $1

-- @GEORGE - no updated at