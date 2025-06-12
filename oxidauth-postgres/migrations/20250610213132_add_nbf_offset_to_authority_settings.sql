UPDATE authorities
SET
    settings = jsonb_set (
        settings,
        '{jwt_nbf_offset}',
        '{"enabled": {"secs": 10, "nanos": 0}}'
    )
WHERE settings ->> 'jwt_nbf_offset' IS NULL;
