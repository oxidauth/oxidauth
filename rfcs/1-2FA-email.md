# RFC: 2FA - Email

### Summary
Enable two factor authentication via email. The user would provide their email/username and password (existing functionality) then be prompted to input a 6 digit code delivered to their email (new functionality) in order to login. The 6 digit code will be generated as an HOTP (HMAC based One Time Password) code with a SHA215 encoding algorithm.

### Benefit
Many companies are now requiring 2FA on all accounts and platforms their employees may use. The addition of this feature would allow companies using oxidauth to comply with this requirement. 

### Technical Flow
From the frontend login screen, a user provides username & password combo, if password is correct:
1. oxidauth generates a secure code with expiration set to 10 min
2. code is stored in a DB table
3. code is also emailed to the users email
4. if 10 minutes pass and the code expires, the code is deleted from the table
5. if a new code is requested by the user, the previous code is deleted from the table (meaning 1 code per user id allowed in the table at a time)
6. if incorrect code is provided, an error message is returned saying the code was not valid
7. if correct code is provided, the sign in process is completed and the user is logged in

### New Endpoints
- [POST] /auth_codes - body supplies a code for verification
- [GET] /auth_codes - requests new code & email 

### Libraries
Code generation library: [otps](https://lib.rs/crates/otps) This package is a library designed to provide out-of-box HOTP and TOTP clients to generate one-time passwords.

### Database Migrations
Table: auth_codes
Columns: id, user_id, code, created_at, expires_at

### Research & Alternatives
- Alternate library [rust-otp](https://github.com/WesleyBatista/rust-otp)
- HOTP spec [hotp RFC](https://datatracker.ietf.org/doc/html/rfc4226)
- a company that did this (gosquared)[https://www.gosquared.com/blog/building-two-factor-authentication]
- a resource - used multiple articles here (onelogin)[https://www.onelogin.com/learn/otp-totp-hotp]

### Outstanding Questions
- Should this be required by default and a part of all log in flows? Or, should there be an opt in variable passed to allow 1FA 
- Is an external library needed to generate an HOTP code, or can a cryptographicaly secure pseudo-random number be used instead

### UI
<img width="743" alt="Screenshot 2024-04-12-login" src="rfcs/images/Screenshot 2024-04-12-email-login.png">

<img width="751" alt="Screenshot 2024-04-12 at 10 46 52 AM" src="rfcs/images/Screenshot 2024-04-12-email-code.png">