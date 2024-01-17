SELECT
    user_role_grants.user_id,
    user_role_grants.role_id,
    user_role_grants.created_at,
    user_role_grants.updated_at,
    roles.name,
    roles.created_at AS role_created_at,
    roles.updated_at AS role_updated_at
FROM user_role_grants
JOIN roles ON user_role_grants.role_id = roles.id
WHERE user_role_grants.user_id = $1
