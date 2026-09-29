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

POST {{scheme}}://{{host}}/api/v1/roles
{
    "role": {
        "name": "firefly:pilot"
    }
}
HTTP/1.1 200
[Captures]
child_id: jsonpath "$.payload.role.id"
[Asserts]
jsonpath "$.success" == true

POST {{scheme}}://{{host}}/api/v1/roles/{{role_id}}/roles/{{child_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload" != null 
jsonpath "$.payload.grant" != null 
jsonpath "$.payload.grant.parent_id" == "{{role_id}}"
jsonpath "$.payload.grant.child_id" == "{{child_id}}"
jsonpath "$.payload.grant.created_at" != null 
jsonpath "$.payload.grant.updated_at" != null 
jsonpath "$.payload.child" != null 
jsonpath "$.payload.child.id" == "{{child_id}}"
jsonpath "$.payload.child.name" == "firefly:pilot"
jsonpath "$.payload.child.created_at" != null 
jsonpath "$.payload.child.updated_at" != null 

DELETE {{scheme}}://{{host}}/api/v1/roles/{{role_id}}/roles/{{child_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists

DELETE {{scheme}}://{{host}}/api/v1/roles/{{child_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists

DELETE {{scheme}}://{{host}}/api/v1/roles/{{role_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
