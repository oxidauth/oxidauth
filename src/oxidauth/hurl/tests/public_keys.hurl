# Public keys: create is permission-guarded (oxidauth:public_keys:create);
# list/find/delete are public so jwt verification can bootstrap itself.

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

# creation without a bearer is rejected by the extractor
POST {{scheme}}://{{host}}/api/v1/public_keys
HTTP/1.1 401

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

# listing is public and includes the bootstrap keypair
GET {{scheme}}://{{host}}/api/v1/public_keys
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.public_keys" count >= 1

GET {{scheme}}://{{host}}/api/v1/public_keys/{{public_key_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.public_key.id" == "{{public_key_id}}"
jsonpath "$.payload.public_key.public_key" == "{{public_key}}"

GET {{scheme}}://{{host}}/api/v1/public_keys/00000000-0000-0000-0000-000000000000
HTTP/1.1 400
[Asserts]
jsonpath "$.success" == false
jsonpath "$.errors[0].debug" contains "RowNotFound"

DELETE {{scheme}}://{{host}}/api/v1/public_keys/{{public_key_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.public_key.id" == "{{public_key_id}}"

GET {{scheme}}://{{host}}/api/v1/public_keys/{{public_key_id}}
HTTP/1.1 400
[Asserts]
jsonpath "$.success" == false
jsonpath "$.errors[0].debug" contains "RowNotFound"
