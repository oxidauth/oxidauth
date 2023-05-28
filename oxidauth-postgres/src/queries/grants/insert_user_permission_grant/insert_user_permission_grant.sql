INSERT INTO user_permission_grants
(user_id, permission_id)
VALUES ($1, $2)
RETURNING *
