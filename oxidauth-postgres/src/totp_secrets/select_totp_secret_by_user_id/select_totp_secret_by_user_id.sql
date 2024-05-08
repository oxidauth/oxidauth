SELECT totp_secret
FROM totp_secrets
WHERE user_id = $1