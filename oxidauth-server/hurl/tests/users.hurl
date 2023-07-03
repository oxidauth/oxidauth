GET {{scheme}}://{{host}}/api/v1/users
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" != null
jsonpath "$.errors" not exists
jsonpath "$.payload.users" != null
jsonpath "$.payload.users" count == 0

POST {{scheme}}://{{host}}/api/v1/authorities
{
    "authority": {
        "name": "default-01",
        "strategy": "username_password",
        "params": {
            "salt": "wry4tb4tj54nt4tjt4231sikiu89SDGBGIp97"
        },
        "settings": {
            "jwt_ttl": {
                "secs": 120,
                "nanos": 0
            },
            "refresh_token_ttl": {
                "secs": 345600,
                "nanos": 0
            }
        }
    }
}
HTTP/1.1 200
[Captures]
authority_id: jsonpath "$.payload.authority.id"
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" != null
jsonpath "$.errors" not exists

GET {{scheme}}://{{host}}/api/v1/users/00000000-0000-0000-0000-000000000000
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == false
jsonpath "$.payload" not exists 
jsonpath "$.errors" != null 
jsonpath "$.errors[0]" contains "user not found"

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
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" != null
jsonpath "$.errors" not exists


GET {{scheme}}://{{host}}/api/v1/users
HTTP/1.1 200
[Captures]
user_id: jsonpath "$.payload.users[0].id" 
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" != null
jsonpath "$.errors" not exists
jsonpath "$.payload.users" != null
jsonpath "$.payload.users" count == 1
jsonpath "$.payload.users[0].id" != null 
jsonpath "$.payload.users[0].username" == "browncoat1" 
jsonpath "$.payload.users[0].email" == "mal@serenity.com" 
jsonpath "$.payload.users[0].first_name" == "Mal" 
jsonpath "$.payload.users[0].last_name" == "Reynolds" 
jsonpath "$.payload.users[0].status" == "active"
jsonpath "$.payload.users[0].kind" == "human"
jsonpath "$.payload.users[0].profile" != null 
jsonpath "$.payload.users[0].created_at" != null 
jsonpath "$.payload.users[0].updated_at" != null 


GET {{scheme}}://{{host}}/api/v1/users/{{user_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" != null
jsonpath "$.errors" not exists
jsonpath "$.payload.user" != null
jsonpath "$.payload.user.id" != null 
jsonpath "$.payload.user.username" == "browncoat1" 
jsonpath "$.payload.user.email" == "mal@serenity.com" 
jsonpath "$.payload.user.first_name" == "Mal" 
jsonpath "$.payload.user.last_name" == "Reynolds" 
jsonpath "$.payload.user.status" == "active"
jsonpath "$.payload.user.kind" == "human"
jsonpath "$.payload.user.profile" != null 
jsonpath "$.payload.user.created_at" != null 
jsonpath "$.payload.user.updated_at" != null 


GET {{scheme}}://{{host}}/api/v1/users/by_username/browncoat1
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" != null
jsonpath "$.errors" not exists
jsonpath "$.payload.user" != null
jsonpath "$.payload.user.id" != null 
jsonpath "$.payload.user.username" == "browncoat1" 
jsonpath "$.payload.user.email" == "mal@serenity.com" 
jsonpath "$.payload.user.first_name" == "Mal" 
jsonpath "$.payload.user.last_name" == "Reynolds" 
jsonpath "$.payload.user.status" == "active"
jsonpath "$.payload.user.kind" == "human"
jsonpath "$.payload.user.profile" != null 
jsonpath "$.payload.user.created_at" != null 
jsonpath "$.payload.user.updated_at" != null 

PUT {{scheme}}://{{host}}/api/v1/users/{{user_id}}
{
    "user": {
        "email": "kaylee@serenity.com",
        "first_name": "Kaylee",
        "last_name": "Frye"
    }
}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" != null
jsonpath "$.errors" not exists
jsonpath "$.payload.user" != null
jsonpath "$.payload.user.id" != null 
jsonpath "$.payload.user.username" == "browncoat1" 
jsonpath "$.payload.user.email" == "kaylee@serenity.com" 
jsonpath "$.payload.user.first_name" == "Kaylee" 
jsonpath "$.payload.user.last_name" == "Frye" 
jsonpath "$.payload.user.status" == "active"
jsonpath "$.payload.user.kind" == "human"
jsonpath "$.payload.user.profile" != null 
jsonpath "$.payload.user.created_at" != null 
jsonpath "$.payload.user.updated_at" != null 

DELETE {{scheme}}://{{host}}/api/v1/users/{{user_id}}/authorities/{{authority_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists

DELETE {{scheme}}://{{host}}/api/v1/authorities/{{authority_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists

DELETE {{scheme}}://{{host}}/api/v1/users/{{user_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
