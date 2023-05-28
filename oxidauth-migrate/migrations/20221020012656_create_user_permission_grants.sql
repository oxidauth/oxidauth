CREATE TABLE user_permission_grants (
    user_id UUID NOT NULL,
    permission_id UUID NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY(user_id, permission_id),

    CONSTRAINT user_permission_grants_users_fk FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT user_permission_grants_permissions_fk FOREIGN KEY(permission_id) REFERENCES permissions(id) ON DELETE CASCADE
);

CREATE INDEX user_permission_grants_user_id_idx ON user_permission_grants(user_id);
CREATE INDEX user_permission_grants_permission_id_idx ON user_permission_grants(permission_id);
