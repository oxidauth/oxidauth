SELECT
    role_role_grants.parent_id,
    role_role_grants.child_id,
    role_role_grants.created_at,
    role_role_grants.updated_at,
    roles.name AS role_name,
    roles.created_at AS role_created_at,
    roles.updated_at AS role_updated_at
FROM role_role_grants
JOIN roles ON role_role_grants.child_id = roles.id
WHERE role_role_grants.parent_id = $1
