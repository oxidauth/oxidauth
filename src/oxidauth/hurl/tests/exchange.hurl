# POST /refresh_tokens (live since plan 07 — the DTO takes ONLY the refresh
# token; the user travels inside the token itself).
# Flow: register -> exchange refresh token -> exchange payload names the very
# user we registered -> cleanup.

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

POST {{scheme}}://{{host}}/api/v1/auth/register
{
    "client_key": "{{admin_client_key}}",
    "params": {
        "username": "hurl-exch-{{stamp}}",
        "password": "hurl-password-1",
        "password_confirmation": "hurl-password-1"
    }
}
HTTP/1.1 200
[Captures]
refresh_token: jsonpath "$.payload.refresh_token"
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.jwt" exists
jsonpath "$.payload.refresh_token" exists

POST {{scheme}}://{{host}}/api/v1/refresh_tokens
{
    "refresh_token": "{{refresh_token}}"
}
HTTP/1.1 200
[Captures]
user_id: jsonpath "$.payload.user_id"
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload.jwt" exists
jsonpath "$.payload.refresh_token" exists
jsonpath "$.payload.user_id" exists

# the exchanged token names the user we just registered
GET {{scheme}}://{{host}}/api/v1/users/{{user_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.user.username" == "hurl-exch-{{stamp}}"

# an unknown refresh token is a missing row -> 400 service error
POST {{scheme}}://{{host}}/api/v1/refresh_tokens
{
    "refresh_token": "00000000-0000-0000-0000-000000000000"
}
HTTP/1.1 400
[Asserts]
jsonpath "$.success" == false
jsonpath "$.errors[0].debug" contains "RowNotFound"

DELETE {{scheme}}://{{host}}/api/v1/users/{{user_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
