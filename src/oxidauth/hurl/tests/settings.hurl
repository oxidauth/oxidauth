# /settings is deliberately open (no bearer guard in plan-09 handlers) and
# save is an UPSERT keyed by `key` — so this file runs clean on both passes
# without deleting anything (there is no DELETE /settings route).

POST {{scheme}}://{{host}}/api/v1/settings
{
    "setting": {
        "key": "hurl_test_{{stamp}}",
        "value": "hurl-value-{{stamp}}"
    }
}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload.setting.key" == "hurl_test_{{stamp}}"
jsonpath "$.payload.setting.value" == "hurl-value-{{stamp}}"
jsonpath "$.payload.setting.created_at" exists
jsonpath "$.payload.setting.updated_at" exists

# PUT is an alias of the same save handler (upsert again)
PUT {{scheme}}://{{host}}/api/v1/settings
{
    "setting": {
        "key": "hurl_test_{{stamp}}",
        "value": "hurl-value-updated-{{stamp}}"
    }
}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.setting.value" == "hurl-value-updated-{{stamp}}"

GET {{scheme}}://{{host}}/api/v1/settings/hurl_test_{{stamp}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.setting.key" == "hurl_test_{{stamp}}"
jsonpath "$.payload.setting.value" == "hurl-value-updated-{{stamp}}"

# a key nobody saved answers as a service error
GET {{scheme}}://{{host}}/api/v1/settings/hurl_test_missing_{{stamp}}
HTTP/1.1 400
[Asserts]
jsonpath "$.success" == false
jsonpath "$.errors[0].debug" contains "SettingNotFoundError"
