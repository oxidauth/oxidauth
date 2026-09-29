POST {{scheme}}://{{host}}/api/v1/public_keys
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload" != null
