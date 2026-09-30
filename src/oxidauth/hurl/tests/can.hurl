# GET /can/{permission} — the bearer's own jwt entitlements answer, so an
# unauthorized user gets 200 + payload false (a "no you can't"), never 401;
# 401 is reserved for routes the jwt extractor guards. A user granted a
# permission must RE-AUTHENTICATE for it to show up in the token.

POST {{scheme}}://{{host}}/api/v1/auth/authenticate
{
    "client_key": "{{admin_client_key}}",
    "params": {
        "username": "{{admin_username}}",
        "password": "{{admin_password}}"
    }
}
HTTP/1.1 200
[Captures]
jwt: jsonpath "$.payload.jwt"
[Asserts]
jsonpath "$.success" == true

# the admin's oxidauth:**:** wildcard covers everything
GET {{scheme}}://{{host}}/api/v1/can/oxidauth:users:read
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" == true
jsonpath "$.notices[0]" == "yes you can"
jsonpath "$.errors" not exists

# a permission nobody holds parses fine and answers false
GET {{scheme}}://{{host}}/api/v1/can/hrc{{stamp}}:thing:read
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" == false
jsonpath "$.warnings[0]" == "no you can't"

# four segments is not a permission shape at all -> 400, plain-string error
GET {{scheme}}://{{host}}/api/v1/can/hrc{{stamp}}:thing:read:extra
Authorization: Bearer {{jwt}}
HTTP/1.1 400
[Asserts]
jsonpath "$.success" == false
jsonpath "$.errors[0]" == "invalid permission"

# a fresh registrant has no entitlements at all
POST {{scheme}}://{{host}}/api/v1/auth/register
{
    "client_key": "{{admin_client_key}}",
    "params": {
        "username": "hurl-can-{{stamp}}",
        "password": "hurl-password-1",
        "password_confirmation": "hurl-password-1"
    }
}
HTTP/1.1 200
[Captures]
poor_jwt: jsonpath "$.payload.jwt"
[Asserts]
jsonpath "$.success" == true

GET {{scheme}}://{{host}}/api/v1/can/hrc{{stamp}}:thing:read
Authorization: Bearer {{poor_jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" == false

# and a guarded route really is 401 for them — envelope with success false
GET {{scheme}}://{{host}}/api/v1/users
Authorization: Bearer {{poor_jwt}}
HTTP/1.1 401
[Asserts]
jsonpath "$.success" == false

# grant the permission (admin), re-authenticate (entitlements are minted into
# the token at authentication time), and the answer flips
POST {{scheme}}://{{host}}/api/v1/permissions/hrc{{stamp}}:thing:read
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true

GET {{scheme}}://{{host}}/api/v1/users/by_username/hurl-can-{{stamp}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Captures]
poor_user_id: jsonpath "$.payload.user.id"
[Asserts]
jsonpath "$.success" == true

POST {{scheme}}://{{host}}/api/v1/users/{{poor_user_id}}/permissions/hrc{{stamp}}:thing:read
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload.permission.realm" == "hrc{{stamp}}"

POST {{scheme}}://{{host}}/api/v1/auth/authenticate
{
    "client_key": "{{admin_client_key}}",
    "params": {
        "username": "hurl-can-{{stamp}}",
        "password": "hurl-password-1"
    }
}
HTTP/1.1 200
[Captures]
rich_jwt: jsonpath "$.payload.jwt"
[Asserts]
jsonpath "$.success" == true

GET {{scheme}}://{{host}}/api/v1/can/hrc{{stamp}}:thing:read
Authorization: Bearer {{rich_jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.payload" == true

# cleanup
DELETE {{scheme}}://{{host}}/api/v1/users/{{poor_user_id}}/permissions/hrc{{stamp}}:thing:read
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true

DELETE {{scheme}}://{{host}}/api/v1/permissions/hrc{{stamp}}:thing:read
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true

DELETE {{scheme}}://{{host}}/api/v1/users/{{poor_user_id}}
Authorization: Bearer {{jwt}}
HTTP/1.1 200
[Asserts]
jsonpath "$.success" == true
