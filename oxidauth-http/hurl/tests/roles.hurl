GET {{scheme}}://{{host}}/api/v1/roles

HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" != null
jsonpath "$.errors" not exists
jsonpath "$.payload.roles" != null
jsonpath "$.payload.roles" count == 0

GET {{scheme}}://{{host}}/api/v1/roles/00000000-0000-0000-0000-000000000000

HTTP/1.1 200
[Asserts]
jsonpath "$.success" == false
jsonpath "$.payload" not exists 
jsonpath "$.errors" != null 
jsonpath "$.errors[0]" contains "role not found"

POST {{scheme}}://{{host}}/api/v1/roles
{
    "role": {
        "name": "firefly:captain"
    }
}

HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" != null
jsonpath "$.errors" not exists


GET {{scheme}}://{{host}}/api/v1/roles

HTTP/1.1 200
[Captures]
role_id: jsonpath "$.payload.roles[0].id" 

[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" != null
jsonpath "$.errors" not exists
jsonpath "$.payload.roles" != null
jsonpath "$.payload.roles" count == 1
jsonpath "$.payload.roles[0].id" != null 
jsonpath "$.payload.roles[0].name" == "firefly:captain" 
jsonpath "$.payload.roles[0].created_at" != null 
jsonpath "$.payload.roles[0].updated_at" != null 


GET {{scheme}}://{{host}}/api/v1/roles/{{role_id}}

HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" != null
jsonpath "$.errors" not exists
jsonpath "$.payload.role" != null
jsonpath "$.payload.role.id" != null 
jsonpath "$.payload.role.name" == "firefly:captain" 
jsonpath "$.payload.role.created_at" != null 
jsonpath "$.payload.role.updated_at" != null 


PUT {{scheme}}://{{host}}/api/v1/roles/{{role_id}}
{
    "role": {
        "name": "firefly:mechanic"
    }
}

HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" != null
jsonpath "$.errors" not exists
jsonpath "$.payload.role" != null
jsonpath "$.payload.role.id" != null 
jsonpath "$.payload.role.name" == "firefly:mechanic" 
jsonpath "$.payload.role.created_at" != null 
jsonpath "$.payload.role.updated_at" != null 


DELETE {{scheme}}://{{host}}/api/v1/roles/{{role_id}}

HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
