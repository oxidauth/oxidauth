# Role<->role (parent/child) grants. POST/DELETE /roles/{parent_id}/roles/{child_id}.

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
    "role": { "name": "hurl-rr-parent-{{stamp}}" }
}
HTTP/1.1 200
[Captures]
parent_id: jsonpath "$.payload.role.id"
[Asserts]
jsonpath "$.success" == true

POST {{scheme}}://{{host}}/api/v1/roles
Authorization: Bearer {{jwt}}
{
    "role": { "name": "hurl-rr-child-{{stamp}}" }
}
HTTP/1.1 200
[Captures]
child_id: jsonpath "$.payload.role.id"
[Asserts]
jsonpath "$.success" == true

POST {{scheme}}://{{host}}/api/v1/roles/{{parent_id}}/roles/{{child_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload.child.id" == "{{child_id}}"
jsonpath "$.payload.child.name" == "hurl-rr-child-{{stamp}}"
jsonpath "$.payload.grant.parent_id" == "{{parent_id}}"
jsonpath "$.payload.grant.child_id" == "{{child_id}}"
jsonpath "$.payload.grant.created_at" exists
jsonpath "$.payload.grant.updated_at" exists

GET {{scheme}}://{{host}}/api/v1/roles/{{parent_id}}/roles
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.roles" count == 1
jsonpath "$.payload.roles[0].role.id" == "{{child_id}}"
jsonpath "$.payload.roles[0].grant.parent_id" == "{{parent_id}}"

DELETE {{scheme}}://{{host}}/api/v1/roles/{{parent_id}}/roles/{{child_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.grant.parent_id" == "{{parent_id}}"
jsonpath "$.payload.grant.child_id" == "{{child_id}}"

DELETE {{scheme}}://{{host}}/api/v1/roles/{{child_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true

DELETE {{scheme}}://{{host}}/api/v1/roles/{{parent_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
