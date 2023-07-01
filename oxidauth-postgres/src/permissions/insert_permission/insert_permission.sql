INSERT INTO permissions
(realm, resource, action, created_at, updated_at)
VALUES ($1, $2, $3, NOW(), NOW())
