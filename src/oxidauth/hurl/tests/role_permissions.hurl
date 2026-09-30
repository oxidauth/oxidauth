# Role<->permission grants. POST/DELETE /roles/{role_id}/permissions/{parts};
# the grant is echoed as {permission, grant}.

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

POST {{scheme}}://{{host}}/api/v1/roles
Authorization: Bearer {{jwt}}
{
    "role": { "name": "hurl-rp-role-{{stamp}}" }
}
HTTP/1.1 200
[Captures]
role_id: jsonpath "$.payload.role.id"
[Asserts]
jsonpath "$.success" == true

POST {{scheme}}://{{host}}/api/v1/permissions/hrrp{{stamp}}:serenity:fly
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Captures]
permission_id: jsonpath "$.payload.permission.id"
[Asserts]
jsonpath "$.success" == true

POST {{scheme}}://{{host}}/api/v1/roles/{{role_id}}/permissions/hrrp{{stamp}}:serenity:fly
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload.permission.id" == "{{permission_id}}"
jsonpath "$.payload.permission.realm" == "hrrp{{stamp}}"
jsonpath "$.payload.permission.resource" == "serenity"
jsonpath "$.payload.permission.action" == "fly"
jsonpath "$.payload.grant.role_id" == "{{role_id}}"
jsonpath "$.payload.grant.permission_id" == "{{permission_id}}"
jsonpath "$.payload.grant.created_at" exists
jsonpath "$.payload.grant.updated_at" exists

GET {{scheme}}://{{host}}/api/v1/roles/{{role_id}}/permissions
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.permissions" count == 1
jsonpath "$.payload.permissions[0].permission.id" == "{{permission_id}}"
jsonpath "$.payload.permissions[0].grant.role_id" == "{{role_id}}"

DELETE {{scheme}}://{{host}}/api/v1/roles/{{role_id}}/permissions/hrrp{{stamp}}:serenity:fly
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.grant.role_id" == "{{role_id}}"
jsonpath "$.payload.grant.permission_id" == "{{permission_id}}"

DELETE {{scheme}}://{{host}}/api/v1/permissions/hrrp{{stamp}}:serenity:fly
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true

DELETE {{scheme}}://{{host}}/api/v1/roles/{{role_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
