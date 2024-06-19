ALTER TABLE user_authorities DROP CONSTRAINT user_authorities_user_identifier_key;

ALTER TABLE user_authorities
ADD UNIQUE (user_identifier, authority_id);