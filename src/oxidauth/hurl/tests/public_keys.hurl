GET {{scheme}}://{{host}}/api/v1/public_keys
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" != null
jsonpath "$.errors" not exists

POST {{scheme}}://{{host}}/api/v1/public_keys
HTTP/1.1 200
[Captures]
public_key_id: jsonpath "$.payload.public_key.id"
public_key: jsonpath "$.payload.public_key.public_key"
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload" != null
jsonpath "$.payload.public_key" != null
jsonpath "$.payload.public_key.id" != null
jsonpath "$.payload.public_key.public_key" != null
jsonpath "$.payload.public_key.created_at" != null
jsonpath "$.payload.public_key.updated_at" != null

GET {{scheme}}://{{host}}/api/v1/public_keys/{{public_key_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload" != null
jsonpath "$.payload.public_key" != null
jsonpath "$.payload.public_key.id" == "{{public_key_id}}"
jsonpath "$.payload.public_key.public_key" == "{{public_key}}"
jsonpath "$.payload.public_key.created_at" != null
jsonpath "$.payload.public_key.updated_at" != null

DELETE {{scheme}}://{{host}}/api/v1/public_keys/{{public_key_id}}
HTTP/1.1 200
