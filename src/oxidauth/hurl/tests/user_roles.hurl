POST {{scheme}}://{{host}}/api/v1/users
{
    "user": {
        "username": "browncoat1",
        "email": "mal@serenity.com",
        "first_name": "Mal",
        "last_name": "Reynolds"
    }
}
HTTP/1.1 200
[Captures]
user_id: jsonpath "$.payload.user.id"
[Asserts]
jsonpath "$.success" == true

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

POST {{scheme}}://{{host}}/api/v1/users/{{user_id}}/roles/{{role_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload" != null 
jsonpath "$.payload.grant" != null 
jsonpath "$.payload.grant.user_id" == "{{user_id}}"
jsonpath "$.payload.grant.role_id" == "{{role_id}}"
jsonpath "$.payload.grant.created_at" != null 
jsonpath "$.payload.grant.updated_at" != null 
jsonpath "$.payload.role" != null 
jsonpath "$.payload.role.id" == "{{role_id}}"
jsonpath "$.payload.role.name" == "firefly:captain"
jsonpath "$.payload.role.created_at" != null 
jsonpath "$.payload.role.updated_at" != null 

DELETE {{scheme}}://{{host}}/api/v1/users/{{user_id}}/roles/{{role_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists

DELETE {{scheme}}://{{host}}/api/v1/roles/{{role_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists

DELETE {{scheme}}://{{host}}/api/v1/users/{{user_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
