SELECT
    user_permission_grants.user_id,
    user_permission_grants.permission_id,
    user_permission_grants.created_at,
    user_permission_grants.updated_at,
    permissions.realm,
    permissions.resource,
    permissions.action,
    permissions.created_at AS permission_created_at,
    permissions.updated_at AS permission_updated_at
FROM user_permission_grants
JOIN permissions ON user_permission_grants.permission_id = permissions.id
WHERE user_permission_grants.user_id = $1
