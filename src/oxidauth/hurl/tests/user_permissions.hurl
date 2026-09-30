# User<->permission grants. POST/DELETE /users/{user_id}/permissions/{parts}.

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
    "user": { "username": "hurl-up-user-{{stamp}}" }
}
HTTP/1.1 200
[Captures]
user_id: jsonpath "$.payload.user.id"
[Asserts]
jsonpath "$.success" == true

POST {{scheme}}://{{host}}/api/v1/permissions/hup{{stamp}}:serenity:fly
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Captures]
permission_id: jsonpath "$.payload.permission.id"
[Asserts]
jsonpath "$.success" == true

GET {{scheme}}://{{host}}/api/v1/users/{{user_id}}/permissions
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.user_permission_grants" count == 0

POST {{scheme}}://{{host}}/api/v1/users/{{user_id}}/permissions/hup{{stamp}}:serenity:fly
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload.permission.id" == "{{permission_id}}"
jsonpath "$.payload.permission.realm" == "hup{{stamp}}"
jsonpath "$.payload.permission.resource" == "serenity"
jsonpath "$.payload.permission.action" == "fly"
jsonpath "$.payload.grant.user_id" == "{{user_id}}"
jsonpath "$.payload.grant.permission_id" == "{{permission_id}}"

GET {{scheme}}://{{host}}/api/v1/users/{{user_id}}/permissions
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.user_permission_grants" count == 1
jsonpath "$.payload.user_permission_grants[0].permission.id" == "{{permission_id}}"
jsonpath "$.payload.user_permission_grants[0].grant.user_id" == "{{user_id}}"

DELETE {{scheme}}://{{host}}/api/v1/users/{{user_id}}/permissions/hup{{stamp}}:serenity:fly
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.user_permission.grant.user_id" == "{{user_id}}"
jsonpath "$.payload.user_permission.permission.id" == "{{permission_id}}"

GET {{scheme}}://{{host}}/api/v1/users/{{user_id}}/permissions
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.user_permission_grants" count == 0

DELETE {{scheme}}://{{host}}/api/v1/permissions/hup{{stamp}}:serenity:fly
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true

DELETE {{scheme}}://{{host}}/api/v1/users/{{user_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
