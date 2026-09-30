# Ensures a jwt-signing keypair exists (bootstrap creates one; this creates an
# extra pair and removes it again, so it is idempotent and self-cleaning).
# Create requires the oxidauth:public_keys:create permission (admin wildcard
# covers it); list/find/delete are deliberately public.

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

POST {{scheme}}://{{host}}/api/v1/public_keys
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Captures]
public_key_id: jsonpath "$.payload.public_key.id"
public_key: jsonpath "$.payload.public_key.public_key"
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload.public_key.id" exists
jsonpath "$.payload.public_key.public_key" exists
jsonpath "$.payload.public_key.created_at" exists
jsonpath "$.payload.public_key.updated_at" exists

GET {{scheme}}://{{host}}/api/v1/public_keys/{{public_key_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.public_key.id" == "{{public_key_id}}"
jsonpath "$.payload.public_key.public_key" == "{{public_key}}"

DELETE {{scheme}}://{{host}}/api/v1/public_keys/{{public_key_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
