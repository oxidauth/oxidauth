# Summary

Enable two factor authentication via email. The user would provide their email/username and password (existing functionality) then be prompted to input a 6 digit code delivered to their email (new functionality) in order to login. The 6 digit code will be generated as an TOTP (Time based One Time Password) code.

### Benefit

Many companies are now requiring 2FA on all accounts and platforms their employees may use. The addition of this feature would allow companies using oxidauth to comply with this requirement.


### Technical Flow
Each user is assigned a static auth key at creation (stored in new table auth_keys). From the frontend login screen, a user provides username & password combo, if password is correct:

- a temporary jwt is sent to the browser, allowing the user to access the 2FA code input screen
- a 6 digit code is created from the secret key + current time, then emailed to the user
- user inputs the 6 digit code from their email which is diffed against the secrety key + current time
- if 5 minutes have passed, the new code will not match the user entered code, and login will fail. Returns an error.
- if user input code does match the new code for any reason, login will fail. Returns an error
- if user requests a new code, the 6 digit code is recreated with new current time, and emailed to the user
- user can continue to try codes until the temporary jwt times out (or, max number of tries is reached, if we implement)
- if user input matches new code, a jwt is returned and user is logged in

### New Endpoints
- [POST] /auth_codes - body supplies a code for verification
- [GET] /auth_codes - requests new code & email

### New Frontend Pages
- Email code entry form

### Other changes
- OxidAuth Authority will now have an added property of `require-2fa`, which, if true, will create a 2fa code for each new user and impact the sign in flow.

### Libraries
Code generation library: [Boring Auth](https://docs.rs/boringauth) This package is a library designed to provide out-of-box HOTP and TOTP clients to generate one-time passwords.

### Database Migrations
Table: auth_keys
Columns: id, user_id, key, created_at, updated_at

### Research & Alternatives
- Alternate library rust-otp, otps, libreauth
- HOTP spec hotp RFC
- a company that did this (gosquared)[https://www.gosquared.com/blog/building-two-factor-authentication]
- a resource - used multiple articles here (onelogin)[https://www.onelogin.com/learn/otp-totp-hotp]

### Outstanding Questions
- addressed in April 16 meeting


### UI
<img width="743" alt="Screenshot 2024-04-12-login" src="./images/rfc1-login.png">

<img width="751" alt="Screenshot 2024-04-12 at 10 46 52 AM" src="./images/rfc1-email-code.png">