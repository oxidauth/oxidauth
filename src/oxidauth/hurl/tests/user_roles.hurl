# User<->role grants. POST/DELETE /users/{user_id}/roles/{role_id}.

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

POST {{scheme}}://{{host}}/api/v1/users
Authorization: Bearer {{jwt}}
{
    "user": { "username": "hurl-ur-user-{{stamp}}" }
}
HTTP/1.1 200
[Captures]
user_id: jsonpath "$.payload.user.id"
[Asserts]
jsonpath "$.success" == true

POST {{scheme}}://{{host}}/api/v1/roles
Authorization: Bearer {{jwt}}
{
    "role": { "name": "hurl-ur-role-{{stamp}}" }
}
HTTP/1.1 200
[Captures]
role_id: jsonpath "$.payload.role.id"
[Asserts]
jsonpath "$.success" == true

GET {{scheme}}://{{host}}/api/v1/users/{{user_id}}/roles
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.user_role_grants" count == 0

POST {{scheme}}://{{host}}/api/v1/users/{{user_id}}/roles/{{role_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload.role.id" == "{{role_id}}"
jsonpath "$.payload.role.name" == "hurl-ur-role-{{stamp}}"
jsonpath "$.payload.grant.user_id" == "{{user_id}}"
jsonpath "$.payload.grant.role_id" == "{{role_id}}"

GET {{scheme}}://{{host}}/api/v1/users/{{user_id}}/roles
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.user_role_grants" count == 1
jsonpath "$.payload.user_role_grants[0].role.id" == "{{role_id}}"
jsonpath "$.payload.user_role_grants[0].grant.user_id" == "{{user_id}}"

DELETE {{scheme}}://{{host}}/api/v1/users/{{user_id}}/roles/{{role_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.user_role.grant.user_id" == "{{user_id}}"
jsonpath "$.payload.user_role.role.id" == "{{role_id}}"

GET {{scheme}}://{{host}}/api/v1/users/{{user_id}}/roles
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.user_role_grants" count == 0

DELETE {{scheme}}://{{host}}/api/v1/roles/{{role_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true

DELETE {{scheme}}://{{host}}/api/v1/users/{{user_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
