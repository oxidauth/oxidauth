CREATE TABLE users (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    username VARCHAR(64) NOT NULL UNIQUE,
    email VARCHAR(256),
    first_name VARCHAR(64),
    last_name VARCHAR(64),
    profile JSONB NOT NULL DEFAULT '{}'::jsonb,
    kind VARCHAR(32) NOT NULL,
    status VARCHAR(32) NOT NULL,
    created_at TIMESTAMPZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX users_username_idx ON users(username);
CREATE INDEX users_email_idx ON users(email);
CREATE INDEX users_kind_idx ON users(kind);
CREATE INDEX users_status_idx ON users(status);
