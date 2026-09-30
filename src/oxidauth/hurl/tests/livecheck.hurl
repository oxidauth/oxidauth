# Plan-07 __meta contract: liveness never touches the database.

GET {{scheme}}://{{host}}/api/v1/__meta/livecheck
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload.version" == "{{api_version}}"
jsonpath "$.payload.healthy" == true
