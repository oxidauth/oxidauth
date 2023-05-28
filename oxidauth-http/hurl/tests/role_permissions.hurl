POST {{scheme}}://{{host}}/api/v1/roles
{
    "role": {
        "name": "firefly:captain"
    }
}
HTTP/1.1 200
[Captures]
role_id: jsonpath "$.payload.role.id"
[Asserts]
jsonpath "$.success" == true

POST {{scheme}}://{{host}}/api/v1/permissions/firefly:serenity:fly
HTTP/1.1 200
[Captures]
permission_id: jsonpath "$.payload.permission.id"
[Asserts]
jsonpath "$.success" == true

POST {{scheme}}://{{host}}/api/v1/roles/{{role_id}}/permissions/firefly:serenity:fly
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload" != null 
jsonpath "$.payload.grant" != null 
jsonpath "$.payload.grant.role_id" == "{{role_id}}"
jsonpath "$.payload.grant.permission_id" == "{{permission_id}}"
jsonpath "$.payload.grant.created_at" != null 
jsonpath "$.payload.grant.updated_at" != null 
jsonpath "$.payload.permission" != null 
jsonpath "$.payload.permission.id" == "{{permission_id}}"
jsonpath "$.payload.permission.realm" == "firefly"
jsonpath "$.payload.permission.resource" == "serenity"
jsonpath "$.payload.permission.action" == "fly"
jsonpath "$.payload.permission.created_at" != null 
jsonpath "$.payload.permission.updated_at" != null 

DELETE {{scheme}}://{{host}}/api/v1/roles/{{role_id}}/permissions/firefly:serenity:fly
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists

DELETE {{scheme}}://{{host}}/api/v1/permissions/firefly:serenity:fly
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists

DELETE {{scheme}}://{{host}}/api/v1/roles/{{role_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
