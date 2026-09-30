# POST /auth/register against the deterministic-bootstrap (username_password)
# authority. The registrant's username embeds {{stamp}}, so the second pass of
# the suite only succeeds if this file's delete actually removed the user.

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

# password and confirmation disagree -> honest 400 service error
POST {{scheme}}://{{host}}/api/v1/auth/register
{
    "client_key": "{{admin_client_key}}",
    "params": {
        "username": "hurl-reg-{{stamp}}",
        "password": "hurl-password-1",
        "password_confirmation": "hurl-password-2"
    }
}
HTTP/1.1 400
[Asserts]
jsonpath "$.success" == false
jsonpath "$.errors[0].debug" contains "password and password confirmation do not match"

# the real registration
POST {{scheme}}://{{host}}/api/v1/auth/register
{
    "client_key": "{{admin_client_key}}",
    "params": {
        "username": "hurl-reg-{{stamp}}",
        "password": "hurl-password-1",
        "password_confirmation": "hurl-password-1",
        "email": "hurl-reg-{{stamp}}@oxidauth.rs",
        "first_name": "Hurl",
        "last_name": "Registrant"
    }
}
HTTP/1.1 200
[Captures]
user_jwt: jsonpath "$.payload.jwt"
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload.jwt" exists
jsonpath "$.payload.refresh_token" exists

# a second registration with the same username hits the unique index
POST {{scheme}}://{{host}}/api/v1/auth/register
{
    "client_key": "{{admin_client_key}}",
    "params": {
        "username": "hurl-reg-{{stamp}}",
        "password": "hurl-password-1",
        "password_confirmation": "hurl-password-1"
    }
}
HTTP/1.1 400
[Asserts]
jsonpath "$.success" == false
jsonpath "$.errors[0].debug" contains "UserAlreadyExistsError"
jsonpath "$.errors[0].display" == "username is already taken"
jsonpath "$.errors[0].debug" not contains "duplicate key"
jsonpath "$.errors[0].debug" not contains "23505"
jsonpath "$.errors[0].debug" not contains "users_username_key"
jsonpath "$.errors[0].source" not exists

# the admin can see the new user
GET {{scheme}}://{{host}}/api/v1/users/by_username/hurl-reg-{{stamp}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Captures]
user_id: jsonpath "$.payload.user.id"
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.user.username" == "hurl-reg-{{stamp}}"
jsonpath "$.payload.user.email" == "hurl-reg-{{stamp}}@oxidauth.rs"
jsonpath "$.payload.user.first_name" == "Hurl"
jsonpath "$.payload.user.last_name" == "Registrant"
jsonpath "$.payload.user.kind" == "human"
jsonpath "$.payload.user.status" == "enabled"

# cleanup — and pass 2 of the suite proves this really deleted the row
DELETE {{scheme}}://{{host}}/api/v1/users/{{user_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.user.id" == "{{user_id}}"

GET {{scheme}}://{{host}}/api/v1/users/{{user_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 400
[Asserts]
jsonpath "$.success" == false
jsonpath "$.errors[0].debug" contains "RowNotFound"
