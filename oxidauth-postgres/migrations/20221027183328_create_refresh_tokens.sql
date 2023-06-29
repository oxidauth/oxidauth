CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,
    authority_id UUID NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT refresh_tokens_users_fk FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT refresh_tokens_authorities_fk FOREIGN KEY(authority_id) REFERENCES authorities(id) ON DELETE CASCADE
);

CREATE INDEX refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX refresh_tokens_authority_id ON refresh_tokens(authority_id);
