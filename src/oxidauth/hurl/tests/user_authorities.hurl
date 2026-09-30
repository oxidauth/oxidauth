# User<->authority attach. POST /users/{user_id}/authorities is keyed by the
# AUTHORITY's client_key (not its id); params are the strategy payload, and
# the username_password registrar stores an argon2 `password_hash`.

POST {{scheme}}://{{host}}/api/v1/auth/authenticate
{
    "client_key": "{{admin_client_key}}",
    "params": {
        "username": "{{admin_username}}",
        "password": "{{admin_password}}"
    }
}
HTTP/1.1 200
[Captures]
jwt: jsonpath "$.payload.jwt"
[Asserts]
jsonpath "$.success" == true

POST {{scheme}}://{{host}}/api/v1/authorities
Authorization: Bearer {{jwt}}
{
    "authority": {
        "name": "hurl-ua-auth-{{stamp}}",
        "strategy": "username_password",
        "params": { "password_salt": "hurl-ua-salt-{{stamp}}" },
        "settings": {
            "jwt_ttl": { "secs": 120, "nanos": 0 },
            "jwt_nbf_offset": "disabled",
            "refresh_token_ttl": { "secs": 345600, "nanos": 0 },
            "totp": "disabled",
            "entitlements_encoding": "txt"
        }
    }
}
HTTP/1.1 200
[Captures]
authority_id: jsonpath "$.payload.authority.id"
authority_client_key: jsonpath "$.payload.authority.client_key"
[Asserts]
jsonpath "$.success" == true

POST {{scheme}}://{{host}}/api/v1/users
Authorization: Bearer {{jwt}}
{
    "user": { "username": "hurl-ua-user-{{stamp}}" }
}
HTTP/1.1 200
[Captures]
user_id: jsonpath "$.payload.user.id"
[Asserts]
jsonpath "$.success" == true

# no attachments yet
GET {{scheme}}://{{host}}/api/v1/users/{{user_id}}/authorities
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.user_authorities" count == 0

POST {{scheme}}://{{host}}/api/v1/users/{{user_id}}/authorities
Authorization: Bearer {{jwt}}
{
    "client_key": "{{authority_client_key}}",
    "user_authority": {
        "params": {
            "username": "hurl-ua-id-{{stamp}}",
            "password": "hurl-ua-password-1",
            "password_confirmation": "hurl-ua-password-1"
        }
    }
}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload.user_authority.user_id" == "{{user_id}}"
jsonpath "$.payload.user_authority.authority_id" == "{{authority_id}}"
jsonpath "$.payload.user_authority.user_identifier" == "hurl-ua-id-{{stamp}}"
jsonpath "$.payload.user_authority.params.password_hash" contains "$argon2"
jsonpath "$.payload.user_authority.created_at" exists
jsonpath "$.payload.user_authority.updated_at" exists

# the attached identity can now authenticate on this authority
POST {{scheme}}://{{host}}/api/v1/auth/authenticate
{
    "client_key": "{{authority_client_key}}",
    "params": {
        "username": "hurl-ua-id-{{stamp}}",
        "password": "hurl-ua-password-1"
    }
}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.jwt" exists

GET {{scheme}}://{{host}}/api/v1/users/{{user_id}}/authorities
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.user_authorities" count == 1
jsonpath "$.payload.user_authorities[0].user_authority.user_id" == "{{user_id}}"
jsonpath "$.payload.user_authorities[0].authority.id" == "{{authority_id}}"

GET {{scheme}}://{{host}}/api/v1/users/{{user_id}}/authorities/{{authority_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.user_authority.authority_id" == "{{authority_id}}"
jsonpath "$.payload.authority.id" == "{{authority_id}}"
jsonpath "$.payload.authority.name" == "hurl-ua-auth-{{stamp}}"
jsonpath "$.payload.authority.strategy" == "username_password"

# update replaces the strategy params payload
PUT {{scheme}}://{{host}}/api/v1/users/{{user_id}}/authorities/{{authority_id}}
Authorization: Bearer {{jwt}}
{
    "params": {
        "username": "hurl-ua-id-{{stamp}}",
        "password": "hurl-ua-password-2"
    }
}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.user_authority.user_id" == "{{user_id}}"
jsonpath "$.payload.user_authority.params" exists

# cleanup (order matters only for readability — deletes cascade)
DELETE {{scheme}}://{{host}}/api/v1/users/{{user_id}}/authorities/{{authority_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.user_authority.authority_id" == "{{authority_id}}"

DELETE {{scheme}}://{{host}}/api/v1/users/{{user_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true

DELETE {{scheme}}://{{host}}/api/v1/authorities/{{authority_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
