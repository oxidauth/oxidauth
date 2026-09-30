# Authority CRUD against the guarded /authorities routes
# (permission oxidauth:authorities:manage, admin wildcard holds it).
# Also proves the deterministic bootstrap seeded the username_password
# authority with the pinned client_key.

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

# no bearer -> extractor rejection, plain 401 (not even the envelope)
GET {{scheme}}://{{host}}/api/v1/authorities
HTTP/1.1 401

# the seeded authority is findable by strategy and carries the pinned client key
GET {{scheme}}://{{host}}/api/v1/authorities/by_strategy/username_password
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.authority.name" == "oxidauth default username_password"
jsonpath "$.payload.authority.client_key" == "{{admin_client_key}}"
jsonpath "$.payload.authority.strategy" == "username_password"
jsonpath "$.payload.authority.status" == "enabled"

# a strategy with no authority answers as a service error, not 404
GET {{scheme}}://{{host}}/api/v1/authorities/by_strategy/oauth2
Authorization: Bearer {{jwt}}
HTTP/1.1 400
[Asserts]
jsonpath "$.success" == false
jsonpath "$.errors[0].debug" contains "Strategy(Oauth2)"

# unknown id -> RowNotFound surfaced as 400 (the api has no 404 mappers)
GET {{scheme}}://{{host}}/api/v1/authorities/00000000-0000-0000-0000-000000000000
Authorization: Bearer {{jwt}}
HTTP/1.1 400
[Asserts]
jsonpath "$.success" == false
jsonpath "$.payload" not exists
jsonpath "$.errors[0].debug" contains "RowNotFound"

# create: settings must be complete (no serde defaults on AuthoritySettings)
POST {{scheme}}://{{host}}/api/v1/authorities
Authorization: Bearer {{jwt}}
{
    "authority": {
        "name": "hurl-auth-{{stamp}}",
        "strategy": "username_password",
        "params": {
            "password_salt": "hurl-salt-{{stamp}}"
        },
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
jsonpath "$.errors" not exists
jsonpath "$.payload.authority.id" exists
jsonpath "$.payload.authority.name" == "hurl-auth-{{stamp}}"
jsonpath "$.payload.authority.client_key" exists
jsonpath "$.payload.authority.status" == "enabled"
jsonpath "$.payload.authority.strategy" == "username_password"
jsonpath "$.payload.authority.settings.jwt_ttl.secs" == 120
jsonpath "$.payload.authority.settings.jwt_nbf_offset" == "disabled"
jsonpath "$.payload.authority.settings.refresh_token_ttl.secs" == 345600
jsonpath "$.payload.authority.settings.totp" == "disabled"
jsonpath "$.payload.authority.settings.entitlements_encoding" == "txt"
jsonpath "$.payload.authority.params.password_salt" == "hurl-salt-{{stamp}}"

# an incomplete settings object is a deserializer error, not a service error
POST {{scheme}}://{{host}}/api/v1/authorities
Authorization: Bearer {{jwt}}
{
    "authority": {
        "name": "hurl-auth-incomplete-{{stamp}}",
        "strategy": "username_password",
        "params": {},
        "settings": {
            "jwt_ttl": { "secs": 120, "nanos": 0 },
            "refresh_token_ttl": { "secs": 345600, "nanos": 0 }
        }
    }
}
HTTP/1.1 422

# a second username_password authority is allowed (the unique-strategy
# constraint was dropped in migration 20240523190325)
PUT {{scheme}}://{{host}}/api/v1/authorities/{{authority_id}}
Authorization: Bearer {{jwt}}
{
    "authority": {
        "name": "hurl-auth-renamed-{{stamp}}",
        "strategy": "username_password",
        "params": {
            "password_salt": "hurl-salt-renamed-{{stamp}}"
        },
        "settings": {
            "jwt_ttl": { "secs": 120, "nanos": 0 },
            "jwt_nbf_offset": { "enabled": { "secs": 10, "nanos": 0 } },
            "refresh_token_ttl": { "secs": 345600, "nanos": 0 },
            "totp": "disabled",
            "entitlements_encoding": "gz"
        }
    }
}
HTTP/1.1 200
# update omits client_key -> the service regenerates it (it does NOT keep the
# create-time value); the new key must exist and need not equal the original
[Captures]
new_client_key: jsonpath "$.payload.authority.client_key"
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.authority.name" == "hurl-auth-renamed-{{stamp}}"
jsonpath "$.payload.authority.settings.jwt_nbf_offset.enabled.secs" == 10
jsonpath "$.payload.authority.settings.entitlements_encoding" == "gz"
jsonpath "$.payload.authority.params.password_salt" == "hurl-salt-renamed-{{stamp}}"

GET {{scheme}}://{{host}}/api/v1/authorities/{{authority_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.authority.id" == "{{authority_id}}"
jsonpath "$.payload.authority.client_key" == "{{new_client_key}}"
jsonpath "$.payload.authority.name" == "hurl-auth-renamed-{{stamp}}"

DELETE {{scheme}}://{{host}}/api/v1/authorities/{{authority_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.authority.id" == "{{authority_id}}"

GET {{scheme}}://{{host}}/api/v1/authorities/{{authority_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 400
[Asserts]
jsonpath "$.success" == false
jsonpath "$.errors[0].debug" contains "RowNotFound"
