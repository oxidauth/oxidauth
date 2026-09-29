SELECT user_authorities.user_id
FROM user_authorities
LEFT JOIN totp_secrets ON user_authorities.user_id = totp_secrets.user_id
WHERE authority_id = $1
AND totp_secrets.id IS NULL