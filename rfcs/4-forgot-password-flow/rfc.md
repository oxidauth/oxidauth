# Forgot password flow

### Summary

Username / Password Authentication forgot password flow to allow users to reset a password they forgot.

### Benefit

In order to be truly useful as a public facing authentication method, the username/password authentication flow must support password reset. This RFC focuses on the changes and additions to the username/password logic required to support this flow.

### New Routes

This flow requires two new routes:

#### 1. POST `/auth/username_password/forgot_password`

Handler file: /src/auth/username_password/forgot_password

Req fields: Email

Res fields: None

_Description_

This route triggers an email send if the email in the request exists in the database. No success or failure response is posted as a security measure to stop account searches by email lists. Email sent will contain a totp code with language that the code expires in 15 min.

#### 2. POST `/auth/username_password/reset_password`

Handler file: /src/auth/username_password/reset_password

Req fields: [New Password, New Password Conf, totp code]

Res fields: [Success, Error]

_Description_

If totp code is valid, this route resets the user password (with salt and pepper same as original password creation).

### Discussion points

- Is it correct for oxidauth to have its own password reset page that all clients use, or should the flow be totally API request based?
- Is it possible and secure to embed the totp code in a "magic link" style url that can be easily handed to the client and on to the user?
