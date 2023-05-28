CREATE TABLE user_authorities (
    user_id UUID NOT NULL,
    authority_id UUID NOT NULL,
    user_identifier VARCHAR(256) UNIQUE NOT NULL,
    params JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY(user_id, authority_id),
    CONSTRAINT user_authorities_users_fk FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT user_authorities_authorities_fk FOREIGN KEY(authority_id) REFERENCES authorities(id) ON DELETE CASCADE
);

CREATE INDEX user_authorities_user_id_idx ON user_authorities(user_id);
CREATE INDEX user_authorities_user_identifier_idx ON user_authorities(user_identifier);
CREATE INDEX user_authorities_authority_id_idx ON user_authorities(authority_id);
