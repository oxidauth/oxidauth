CREATE TABLE users (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    username VARCHAR(32) NOT NULL UNIQUE,
    email VARCHAR(127),
    first_name VARCHAR(32),
    last_name VARCHAR(32),
    profile JSONB NOT NULL DEFAULT '{}'::jsonb,
    kind VARCHAR(32) NOT NULL,
    status VARCHAR(32) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX users_username_idx ON users(username);
CREATE INDEX users_email_idx ON users(email);
CREATE INDEX users_kind_idx ON users(kind);
CREATE INDEX users_status_idx ON users(status);
