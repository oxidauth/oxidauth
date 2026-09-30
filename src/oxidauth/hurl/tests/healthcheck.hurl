# Plan-07 __meta contract: GET /api/v1/__meta/healthcheck answers without auth
# and echoes the API crate's version (api_version injected by hurl.sh).

GET {{scheme}}://{{host}}/api/v1/__meta/healthcheck
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload.version" == "{{api_version}}"
jsonpath "$.payload.healthy" == true
