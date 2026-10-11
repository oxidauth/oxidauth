DELETE FROM oauth_states
WHERE state_hash = $1
  AND client_key = $2
  AND expires_at > NOW()
RETURNING state_hash
