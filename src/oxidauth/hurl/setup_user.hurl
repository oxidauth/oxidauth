# STANDALONE — hurl.sh runs only public_keys_create.hurl + tests/*.hurl;
# invoke me directly: hurl --test <hurl.sh's --variable args> hurl/setup_user.hurl

# Plan-11 bootstrap sanity (run standalone or implicitly as suite context):
# the deterministic bootstrap must expose a usable superuser.

# authenticate as the seeded admin
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
jsonpath "$.errors" not exists
jsonpath "$.payload.jwt" exists
jsonpath "$.payload.refresh_token" exists

# the admin user exists and carries the API kind + enabled status
GET {{scheme}}://{{host}}/api/v1/users/by_username/{{admin_username}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload.user.username" == "{{admin_username}}"
jsonpath "$.payload.user.kind" == "api"
jsonpath "$.payload.user.status" == "enabled"

# the admin's wildcard permission answers yes
GET {{scheme}}://{{host}}/api/v1/can/oxidauth:users:read
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" == true
jsonpath "$.notices[0]" == "yes you can"
