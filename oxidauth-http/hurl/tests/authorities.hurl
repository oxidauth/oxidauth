GET {{scheme}}://{{host}}/api/v1/authorities
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" != null
jsonpath "$.errors" not exists
jsonpath "$.payload.authorities" != null
jsonpath "$.payload.authorities" count == 0

GET {{scheme}}://{{host}}/api/v1/authorities/00000000-0000-0000-0000-000000000000
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == false
jsonpath "$.payload" not exists 
jsonpath "$.errors" != null 
jsonpath "$.errors[0]" contains "authority not found"

POST {{scheme}}://{{host}}/api/v1/authorities
{
    "authority": {
        "name": "default",
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
jsonpath "$.errors" not exists
jsonpath "$.payload" != null
jsonpath "$.payload.authority" != null
jsonpath "$.payload.authority.id" != null
jsonpath "$.payload.authority.name" == "default"
jsonpath "$.payload.authority.strategy" == "username_password"
jsonpath "$.payload.authority.params" != null
jsonpath "$.payload.authority.params.salt" == "wry4tb4tj54nt4tjt4231sikiu89SDGBGIp97"

PUT {{scheme}}://{{host}}/api/v1/authorities/{{authority_id}}
{
    "authority": {
        "name": "renamed",
        "strategy": "username_password",
        "params": {
            "salt": "saltysalt"
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
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload" != null
jsonpath "$.payload.authority" != null
jsonpath "$.payload.authority.id" != null
jsonpath "$.payload.authority.name" == "renamed"
jsonpath "$.payload.authority.strategy" == "username_password"
jsonpath "$.payload.authority.params" != null
jsonpath "$.payload.authority.params.salt" == "saltysalt"

GET {{scheme}}://{{host}}/api/v1/authorities/{{authority_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload" != null
jsonpath "$.payload.authority" != null
jsonpath "$.payload.authority.id" != null
jsonpath "$.payload.authority.name" == "renamed"
jsonpath "$.payload.authority.strategy" == "username_password"
jsonpath "$.payload.authority.params" != null
jsonpath "$.payload.authority.params.salt" == "saltysalt"

DELETE {{scheme}}://{{host}}/api/v1/authorities/{{authority_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
