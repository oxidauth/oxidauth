CREATE TABLE invitations (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX invitations_user_id ON invitations(user_id);
