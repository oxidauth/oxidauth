CREATE TABLE auth_keys (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    user_id UUID FOREIGN KEY REFERENCES Users(id) NOT NULL ON UPDATE cascade ON DELETE cascade,
    key INTEGER[] NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX auth_keys_user_id ON auth_keys(user_id);
