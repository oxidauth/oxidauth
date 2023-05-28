CREATE TABLE role_permission_grants (
    role_id UUID NOT NULL,
    permission_id UUID NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY(role_id, permission_id),

    CONSTRAINT role_permission_grants_roles_fk FOREIGN KEY(role_id) REFERENCES roles(id) ON DELETE CASCADE,
    CONSTRAINT role_permission_grants_permissions_fk FOREIGN KEY(permission_id) REFERENCES permissions(id) ON DELETE CASCADE
);

CREATE INDEX role_permission_grants_role_id_idx ON role_permission_grants(role_id);
CREATE INDEX role_permission_grants_permission_id_idx ON role_permission_grants(permission_id);
