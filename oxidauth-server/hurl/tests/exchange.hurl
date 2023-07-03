POST {{scheme}}://{{host}}/api/v1/authorities
{
    "authority": {
        "name": "default",
        "strategy": "username_password",
        "params": {
            "password_salt": "wry4tb4tj54nt4tjt4231sikiu89SDGBGIp97"
        },
        "settings": {
            "jwt_ttl": {
                "secs": 120,
                "nanos": 0
            },
            "refresh_token_ttl": {
                "secs": 345600,
                "nanos": 0
            }
        }
    }
}
HTTP/1.1 200
[Captures]
authority_id: jsonpath "$.payload.authority.id"
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload" != null


POST {{scheme}}://{{host}}/api/v1/public_keys
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload" != null

POST {{scheme}}://{{host}}/api/v1/register
{
    "strategy": "username_password",
    "params": {
        "username": "malreynolds",
        "first_name": "Mal",
        "last_name": "Reynolds",
        "email": "mreynolds@serenity.com",
        "password": "password123",
        "password_confirmation": "password123"
    }
}
HTTP/1.1 200
[Captures]
jwt: jsonpath "$.payload.jwt"
refresh_token: jsonpath "$.payload.refresh_token"
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload" != null
jsonpath "$.payload.jwt" != null
jsonpath "$.payload.refresh_token" != null

GET {{scheme}}://{{host}}/api/v1/users/by_username/malreynolds
HTTP/1.1 200
[Captures]
user_id: jsonpath "$.payload.user.id"
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload" != null

POST {{scheme}}://{{host}}/api/v1/refresh_tokens
{
    "user_id": "{{user_id}}",
    "refresh_token": "{{refresh_token}}"
}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload" != null
jsonpath "$.payload.jwt" != null
jsonpath "$.payload.refresh_token" != null

DELETE {{scheme}}://{{host}}/api/v1/users/{{user_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists

DELETE {{scheme}}://{{host}}/api/v1/authorities/{{authority_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
