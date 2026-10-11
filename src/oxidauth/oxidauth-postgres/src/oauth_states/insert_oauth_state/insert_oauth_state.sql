-- the same call garbage-collects every expired nonce; data-modifying CTEs run
-- exactly once regardless of whether the outer statement references them
WITH gc AS (
    DELETE FROM oauth_states
    WHERE expires_at < NOW()
)
INSERT INTO oauth_states (state_hash, client_key, expires_at)
VALUES ($1, $2, $3)
