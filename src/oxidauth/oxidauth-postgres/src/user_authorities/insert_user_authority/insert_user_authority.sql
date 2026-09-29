INSERT INTO user_authorities
(user_id, authority_id, user_identifier, params, created_at, updated_at)
VALUES ($1, $2, $3, $4, NOW(), NOW())
RETURNING *
