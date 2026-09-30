# Permission routes key everything off the "{realm}:{resource}:{action}"
# path segment; missing rows answer 400 with PermissionNotFoundError.

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

GET {{scheme}}://{{host}}/api/v1/permissions/hrp{{stamp}}:serenity:fly
Authorization: Bearer {{jwt}}
HTTP/1.1 400
[Asserts]
jsonpath "$.success" == false
jsonpath "$.payload" not exists
jsonpath "$.errors[0].debug" contains "PermissionNotFoundError"
jsonpath "$.errors[0].debug" contains "hrp{{stamp}}:serenity:fly"

POST {{scheme}}://{{host}}/api/v1/permissions/hrp{{stamp}}:serenity:fly
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Captures]
permission_id: jsonpath "$.payload.permission.id"
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload.permission.id" exists
jsonpath "$.payload.permission.realm" == "hrp{{stamp}}"
jsonpath "$.payload.permission.resource" == "serenity"
jsonpath "$.payload.permission.action" == "fly"
jsonpath "$.payload.permission.created_at" exists
jsonpath "$.payload.permission.updated_at" exists

# the unique (realm, resource, action) index rejects duplicates
POST {{scheme}}://{{host}}/api/v1/permissions/hrp{{stamp}}:serenity:fly
Authorization: Bearer {{jwt}}
HTTP/1.1 400
[Asserts]
jsonpath "$.success" == false
jsonpath "$.errors[0].debug" contains "duplicate key"

GET {{scheme}}://{{host}}/api/v1/permissions/hrp{{stamp}}:serenity:fly
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.permission.id" == "{{permission_id}}"
jsonpath "$.payload.permission.realm" == "hrp{{stamp}}"

# the seeded admin permissions are always in the list
# (bootstrap seeds oxidauth:**:*, oxidauth:totp_code:validate and
# oxidauth:auth:forgot_password)
GET {{scheme}}://{{host}}/api/v1/permissions
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.permissions" count >= 2

DELETE {{scheme}}://{{host}}/api/v1/permissions/hrp{{stamp}}:serenity:fly
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.permission.id" == "{{permission_id}}"

GET {{scheme}}://{{host}}/api/v1/permissions/hrp{{stamp}}:serenity:fly
Authorization: Bearer {{jwt}}
HTTP/1.1 400
[Asserts]
jsonpath "$.success" == false
jsonpath "$.errors[0].debug" contains "PermissionNotFoundError"
