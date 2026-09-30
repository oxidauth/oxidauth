# POST /auth/authenticate against the deterministic-bootstrap authority.
# Wrong credentials and unknown client keys are honest 400s (service errors),
# not 401s — 401 only guards bearer-protected routes.
# Creates nothing; safe to run any number of times.

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

# the jwt unlocks a permission-guarded route
GET {{scheme}}://{{host}}/api/v1/users/by_username/{{admin_username}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.user.username" == "{{admin_username}}"

# the same route without a bearer token is rejected outright (empty body)
GET {{scheme}}://{{host}}/api/v1/users/by_username/{{admin_username}}
HTTP/1.1 401

# a wrong password fails as a service error (400 + errors envelope)
POST {{scheme}}://{{host}}/api/v1/auth/authenticate
{
    "client_key": "{{admin_client_key}}",
    "params": {
        "username": "{{admin_username}}",
        "password": "definitely-not-the-admin-password"
    }
}
HTTP/1.1 400
[Asserts]
jsonpath "$.success" == false
jsonpath "$.payload" not exists
jsonpath "$.errors[0].debug" contains "invalid password"

# an unknown client key finds no authority row
POST {{scheme}}://{{host}}/api/v1/auth/authenticate
{
    "client_key": "00000000-0000-0000-0000-000000000000",
    "params": {
        "username": "{{admin_username}}",
        "password": "{{admin_password}}"
    }
}
HTTP/1.1 400
[Asserts]
jsonpath "$.success" == false
jsonpath "$.errors[0].display" contains "authority not found by client_key"

# a malformed uuid is rejected by the deserializer (422), before any handler runs
POST {{scheme}}://{{host}}/api/v1/auth/authenticate
{
    "client_key": "not-a-uuid",
    "params": {
        "username": "{{admin_username}}",
        "password": "{{admin_password}}"
    }
}
HTTP/1.1 422
