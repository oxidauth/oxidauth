SELECT
    user_authorities.user_id,
    user_authorities.authority_id,
    user_authorities.user_identifier,
    user_authorities.params,
    user_authorities.created_at,
    user_authorities.updated_at,
    authorities.name AS authority_name,
    authorities.client_key AS authority_client_key,
    authorities.status AS authority_status,
    authorities.strategy AS authority_strategy,
    authorities.settings AS authority_settings,
    authorities.params AS authority_params,
    authorities.created_at AS authority_created_at,
    authorities.updated_at AS authority_updated_at
FROM user_authorities
JOIN authorities ON user_authorities.authority_id = authorities.id
WHERE user_authorities.user_id = $1
AND user_authorities.authority_id = $2
