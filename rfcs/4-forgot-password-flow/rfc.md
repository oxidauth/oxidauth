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

### File Structure

#### New handler & router files added http

- oxidauth-http/src/server/api/v1/auth/username_password folder
- oxidauth-http/src/server/api/v1/auth/username_password/forgot_password.rs
- oxidauth-http/src/server/api/v1/auth/username_password/reset_password.rs

#### New type files added to Kernel

- oxidauth-kernel/src/auth/username_password folder
- oxidauth-kernel/src/auth/username_password/forgot_password.rs
- oxidauth-kernel/src/auth/username_password/reset_password.rs

#### New UseCase files added

- oxidauth-usecases/src/auth/strategies/username_password folder
- oxidauth-usecases/src/auth/strategies/username_password/forgot_password.rs
- oxidauth-usecases/src/auth/strategies/username_password/reset_password.rs

#### Provider references

- username_password_forgot_password_service
- username_password_reset_password_service

### Discussion points

- @George - review general file structure, knowing that not everything is fleshed out yet so errors will exist
- @George - Is it correct for the Navi API to receive the forgot password request, check that the user exists in Navi, and then request the totp code for the user by id? If no user is found in Navi, no point in reaching out to oxidauth. And inversely, if a user is found in Navi, it seems like Navi should send the user's id to oxidauth. That way Navi is responsible for sending the email (can be branded to Navi's needs) and oxidauth is only responsible for generating the code, resetting refresh tokens, etc..
