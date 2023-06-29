CREATE TABLE role_role_grants (
    parent_id UUID NOT NULL,
    child_id UUID NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY(parent_id, child_id),

    CONSTRAINT parent_role_grants_roles_fk FOREIGN KEY(parent_id) REFERENCES roles(id) ON DELETE CASCADE,
    CONSTRAINT child_role_grants_roles_fk FOREIGN KEY(child_id) REFERENCES roles(id) ON DELETE CASCADE
);

CREATE INDEX role_role_grants_parent_id_idx ON role_role_grants(parent_id);
CREATE INDEX role_role_grants_child_id_idx ON role_role_grants(child_id);
