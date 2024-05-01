CREATE TABLE totp_secrets (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    user_id UUID FOREIGN KEY REFERENCES Users(id) NOT NULL ON UPDATE cascade ON DELETE cascade,
    totp_secret INTEGER[] NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX totp_secrets_user_id ON totp_secrets(user_id);
