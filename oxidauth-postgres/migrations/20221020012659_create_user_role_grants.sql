CREATE TABLE user_role_grants (
    user_id UUID NOT NULL,
    role_id UUID NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY(user_id, role_id),

    CONSTRAINT user_role_grants_users_fk FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT user_role_grants_roles_fk FOREIGN KEY(role_id) REFERENCES roles(id) ON DELETE CASCADE
);

CREATE INDEX user_role_grants_user_id_idx ON user_role_grants(user_id);
CREATE INDEX user_role_grants_role_id_idx ON user_role_grants(role_id);
