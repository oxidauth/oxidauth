# Role CRUD. Role names are globally unique; the deterministic bootstrap
# seeded oxidauth:admin and the admin user holds it.

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

# the seeded admin role answers by name
GET {{scheme}}://{{host}}/api/v1/roles/by_name/oxidauth:admin
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.role.name" == "oxidauth:admin"

GET {{scheme}}://{{host}}/api/v1/roles/00000000-0000-0000-0000-000000000000
Authorization: Bearer {{jwt}}
HTTP/1.1 400
[Asserts]
jsonpath "$.success" == false
jsonpath "$.payload" not exists
jsonpath "$.errors[0].debug" contains "RowNotFound"

POST {{scheme}}://{{host}}/api/v1/roles
Authorization: Bearer {{jwt}}
{
    "role": {
        "name": "hurl-role-{{stamp}}"
    }
}
HTTP/1.1 200
[Captures]
role_id: jsonpath "$.payload.role.id"
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload.role.id" exists
jsonpath "$.payload.role.name" == "hurl-role-{{stamp}}"
jsonpath "$.payload.role.created_at" exists
jsonpath "$.payload.role.updated_at" exists

# duplicate name -> unique-index violation, honest 400
POST {{scheme}}://{{host}}/api/v1/roles
Authorization: Bearer {{jwt}}
{
    "role": {
        "name": "hurl-role-{{stamp}}"
    }
}
HTTP/1.1 400
[Asserts]
jsonpath "$.success" == false
jsonpath "$.errors[0].debug" contains "duplicate key"

GET {{scheme}}://{{host}}/api/v1/roles/{{role_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.role.id" == "{{role_id}}"

PUT {{scheme}}://{{host}}/api/v1/roles/{{role_id}}
Authorization: Bearer {{jwt}}
{
    "role": {
        "name": "hurl-role-renamed-{{stamp}}"
    }
}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.role.id" == "{{role_id}}"
jsonpath "$.payload.role.name" == "hurl-role-renamed-{{stamp}}"

GET {{scheme}}://{{host}}/api/v1/roles/by_name/hurl-role-renamed-{{stamp}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.role.id" == "{{role_id}}"

DELETE {{scheme}}://{{host}}/api/v1/roles/{{role_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.role.id" == "{{role_id}}"

GET {{scheme}}://{{host}}/api/v1/roles/{{role_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 400
[Asserts]
jsonpath "$.success" == false
jsonpath "$.errors[0].debug" contains "RowNotFound"
