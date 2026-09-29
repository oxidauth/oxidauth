SELECT
    role_permission_grants.role_id,
    role_permission_grants.permission_id,
    role_permission_grants.created_at,
    role_permission_grants.updated_at,
    permissions.realm,
    permissions.resource,
    permissions.action,
    permissions.created_at AS permission_created_at,
    permissions.updated_at AS permission_updated_at
FROM role_permission_grants
JOIN permissions ON role_permission_grants.permission_id = permissions.id
WHERE role_permission_grants.role_id = $1
