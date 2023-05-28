GET {{scheme}}://{{host}}/api/v1/permissions

HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" != null
jsonpath "$.errors" not exists
jsonpath "$.payload.permissions" != null
jsonpath "$.payload.permissions" count == 0

GET {{scheme}}://{{host}}/api/v1/permissions/firefly:serenity:start

HTTP/1.1 200
[Asserts]
jsonpath "$.success" == false
jsonpath "$.payload" not exists 
jsonpath "$.errors" != null 
jsonpath "$.errors[0]" contains "permission not found"

POST {{scheme}}://{{host}}/api/v1/permissions/firefly:serenity:start

HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" != null
jsonpath "$.errors" not exists


GET {{scheme}}://{{host}}/api/v1/permissions

HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" != null
jsonpath "$.errors" not exists
jsonpath "$.payload.permissions" != null
jsonpath "$.payload.permissions" count == 1
jsonpath "$.payload.permissions[0].realm" == "firefly" 
jsonpath "$.payload.permissions[0].resource" == "serenity" 
jsonpath "$.payload.permissions[0].action" == "start" 
jsonpath "$.payload.permissions[0].created_at" != null 
jsonpath "$.payload.permissions[0].updated_at" != null 


GET {{scheme}}://{{host}}/api/v1/permissions/firefly:serenity:start

HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" != null
jsonpath "$.errors" not exists
jsonpath "$.payload.permission" != null
jsonpath "$.payload.permission.realm" == "firefly" 
jsonpath "$.payload.permission.resource" == "serenity" 
jsonpath "$.payload.permission.action" == "start" 
jsonpath "$.payload.permission.created_at" != null 
jsonpath "$.payload.permission.updated_at" != null 


DELETE {{scheme}}://{{host}}/api/v1/permissions/firefly:serenity:start

HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
