# User CRUD (+ by_username / by_ids). The suite's first-pass leftovers would
# make the unique username collide, so the cleanup deletes are load-bearing.

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

# guarded route without a token
GET {{scheme}}://{{host}}/api/v1/users
HTTP/1.1 401

# a uuid the table will never contain
GET {{scheme}}://{{host}}/api/v1/users/00000000-0000-0000-0000-000000000000
Authorization: Bearer {{jwt}}
HTTP/1.1 400
[Asserts]
jsonpath "$.success" == false
jsonpath "$.payload" not exists
jsonpath "$.errors[0].debug" contains "RowNotFound"

GET {{scheme}}://{{host}}/api/v1/users/by_username/hurl-nobody-{{stamp}}
Authorization: Bearer {{jwt}}
HTTP/1.1 400
[Asserts]
jsonpath "$.success" == false
jsonpath "$.errors[0].debug" contains "Username"

POST {{scheme}}://{{host}}/api/v1/users
Authorization: Bearer {{jwt}}
{
    "user": {
        "username": "hurl-user-{{stamp}}",
        "email": "hurl-user-{{stamp}}@oxidauth.rs",
        "first_name": "Hurl",
        "last_name": "User"
    }
}
HTTP/1.1 200
[Captures]
user_id: jsonpath "$.payload.user.id"
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload.user.id" exists
jsonpath "$.payload.user.kind" == "human"
jsonpath "$.payload.user.status" == "enabled"
jsonpath "$.payload.user.username" == "hurl-user-{{stamp}}"
jsonpath "$.payload.user.email" == "hurl-user-{{stamp}}@oxidauth.rs"
jsonpath "$.payload.user.first_name" == "Hurl"
jsonpath "$.payload.user.last_name" == "User"
jsonpath "$.payload.user.profile" exists
jsonpath "$.payload.user.created_at" exists
jsonpath "$.payload.user.updated_at" exists

# duplicate username hits the unique index
POST {{scheme}}://{{host}}/api/v1/users
Authorization: Bearer {{jwt}}
{
    "user": { "username": "hurl-user-{{stamp}}" }
}
HTTP/1.1 400
[Asserts]
jsonpath "$.success" == false
jsonpath "$.errors[0].debug" contains "UserAlreadyExistsError"
jsonpath "$.errors[0].display" == "username is already taken"
jsonpath "$.errors[0].debug" not contains "duplicate key"
jsonpath "$.errors[0].debug" not contains "users_username_key"

GET {{scheme}}://{{host}}/api/v1/users/{{user_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.user.id" == "{{user_id}}"
jsonpath "$.payload.user.username" == "hurl-user-{{stamp}}"

GET {{scheme}}://{{host}}/api/v1/users/by_username/hurl-user-{{stamp}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.user.id" == "{{user_id}}"

# the seeded admin plus this file's user are both listed
GET {{scheme}}://{{host}}/api/v1/users
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.users" count >= 2

# by_ids returns found users and names the missing ids
POST {{scheme}}://{{host}}/api/v1/users/by_ids
Authorization: Bearer {{jwt}}
{
    "user_ids": [
        "{{user_id}}",
        "00000000-0000-0000-0000-000000000000"
    ]
}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.users" count == 1
jsonpath "$.payload.users[0].id" == "{{user_id}}"
jsonpath "$.payload.user_ids_not_found[0]" == "00000000-0000-0000-0000-000000000000"

PUT {{scheme}}://{{host}}/api/v1/users/{{user_id}}
Authorization: Bearer {{jwt}}
{
    "user": {
        "email": "hurl-renamed-{{stamp}}@oxidauth.rs",
        "first_name": "Renamed",
        "last_name": "Crewmember"
    }
}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.user.id" == "{{user_id}}"
jsonpath "$.payload.user.username" == "hurl-user-{{stamp}}"
jsonpath "$.payload.user.email" == "hurl-renamed-{{stamp}}@oxidauth.rs"
jsonpath "$.payload.user.first_name" == "Renamed"
jsonpath "$.payload.user.last_name" == "Crewmember"
jsonpath "$.payload.user.status" == "enabled"

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
