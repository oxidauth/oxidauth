# Summary

Control JWT nbf (Not Before) claims via an authority setting.

### Benefit

The Not Before claim on a JWT ensures that the token is not used before a specified time. In combination with a token expiration date, these settings protect against a variety of attacks and authentication misuses. The current default setting for the nbf claim in oxidauth is 0 - meaning the token is not valid until the exact millisecond specified in the token. This is not compatible with some larger (especially windows based) networks, which may see device times drift away from gobal by time by multiple minutes. This means that even if the global time has passed the nbf time, the device time may be two minutes behind, causing the nbf time to appear not valid and authentication will fail. This issue has been noted with multiple oxidauth clients, many of whom do not have the ability to change the time on their managed device. By moving the nbf claim value to a setting on the oxidauth authority, we can ensure that the nbf is customized to a value that both functions for and proctects the user.

### New Authority setting
Authorities --> Settings --> "jwt_nbf": Duration (eg: { "secs": 900, "nanos": 0 })


### Code changes
When a jwt is built using the JWT builder, the `with_not_before_from` method should now reference the authority setting nbf value instead of the current default of 0.

### Database Migrations
None
