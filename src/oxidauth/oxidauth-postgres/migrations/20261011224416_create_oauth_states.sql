-- OXA-000010: single-use OAuth2 state nonces.
-- The raw state never lands in the database — only its sha256 digest, which
-- is the lookup key for the atomic single-use consume. No FK on client_key:
-- states die within minutes, and an FK would couple authority deletes to
-- in-flight login flows.
CREATE TABLE oauth_states (
    state_hash BYTEA PRIMARY KEY,
    client_key UUID NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
);

-- serves the lazy garbage collection run alongside every insert
CREATE INDEX oauth_states_expires_at_idx ON oauth_states (expires_at);
