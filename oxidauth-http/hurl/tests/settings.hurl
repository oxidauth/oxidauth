POST {{scheme}}://{{host}}/api/v1/settings
{
    "setting": {
        "key": "test_setting",
        "value": "test_setting_value"
    }
}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.errors" not exists
jsonpath "$.payload" != null
jsonpath "$.payload.setting" != null
jsonpath "$.payload.setting.key" == "test_setting"
jsonpath "$.payload.setting.value" == "test_setting_value"
