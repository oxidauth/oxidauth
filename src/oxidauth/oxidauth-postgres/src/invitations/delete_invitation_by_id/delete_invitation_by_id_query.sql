DELETE FROM invitations
WHERE id = $1
RETURNING *
