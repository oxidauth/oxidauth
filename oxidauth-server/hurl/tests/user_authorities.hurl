# create user
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
jsonpath "$.payload" != null
jsonpath "$.errors" not exists

# create authority
POST {{scheme}}://{{host}}/api/v1/authorities
{
    "authority": {
        "name": "default",
        "strategy": "username_password",
        "params": {
            "password_salt": "wry4tb4tj54nt4tjt4231sikiu89SDGBGIp97"
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
jsonpath "$.errors" not exists
jsonpath "$.payload" != null

# empty user authorities
GET {{scheme}}://{{host}}/api/v1/users/{{user_id}}/authorities
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload" != null

# create user_authority
POST {{scheme}}://{{host}}/api/v1/users/{{user_id}}/authorities
{
    "authority_strategy": "username_password",
    "user_authority": {
        "params": {
            "username": "browncoat1",
            "password": "malsunguessablepassword",
            "password_confirmation": "malsunguessablepassword"
        }
    }
}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload" != null
jsonpath "$.payload.user_authority" != null
jsonpath "$.payload.user_authority.user_id" == "{{user_id}}"
jsonpath "$.payload.user_authority.authority_id" == "{{authority_id}}"
jsonpath "$.payload.user_authority.params" != null
jsonpath "$.payload.user_authority.params.password_hash" != null 
jsonpath "$.payload.user_authority.created_at" != null
jsonpath "$.payload.user_authority.updated_at" != null

GET {{scheme}}://{{host}}/api/v1/users/{{user_id}}/authorities/{{authority_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload" != null
jsonpath "$.payload.user_authority.user_id" == "{{user_id}}"
jsonpath "$.payload.user_authority.authority_id" == "{{authority_id}}"
jsonpath "$.payload.user_authority.params" != null
jsonpath "$.payload.user_authority.params.password_hash" != null 
jsonpath "$.payload.user_authority.created_at" != null
jsonpath "$.payload.user_authority.updated_at" != null
jsonpath "$.payload.authority.id" == "{{authority_id}}"
jsonpath "$.payload.authority.name" == "default"
jsonpath "$.payload.authority.strategy" == "username_password"
jsonpath "$.payload.authority.created_at" != null
jsonpath "$.payload.authority.updated_at" != null

GET {{scheme}}://{{host}}/api/v1/users/{{user_id}}/authorities
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload" != null
jsonpath "$.payload.user_authorities[0].user_authority.user_id" == "{{user_id}}"
jsonpath "$.payload.user_authorities[0].user_authority.authority_id" == "{{authority_id}}"
jsonpath "$.payload.user_authorities[0].user_authority.params" != null
jsonpath "$.payload.user_authorities[0].user_authority.params.password_hash" != null
jsonpath "$.payload.user_authorities[0].user_authority.created_at" != null
jsonpath "$.payload.user_authorities[0].user_authority.updated_at" != null
jsonpath "$.payload.user_authorities[0].authority.id" == "{{authority_id}}"
jsonpath "$.payload.user_authorities[0].authority.name" == "default"
jsonpath "$.payload.user_authorities[0].authority.strategy" == "username_password"
jsonpath "$.payload.user_authorities[0].authority.created_at" != null
jsonpath "$.payload.user_authorities[0].authority.updated_at" != null

# delete user_authority
DELETE {{scheme}}://{{host}}/api/v1/users/{{user_id}}/authorities/{{authority_id}}
HTTP/1.1 200

# delete authority
DELETE {{scheme}}://{{host}}/api/v1/authorities/{{authority_id}}
HTTP/1.1 200

# delete user
DELETE {{scheme}}://{{host}}/api/v1/users/{{user_id}}
HTTP/1.1 200
