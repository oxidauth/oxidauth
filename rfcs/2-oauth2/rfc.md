# Summary

Add Oauth2 as an authentication strategy supported by Oxidauth.

### Benefit

Oauth2 is a popular authentication standard that Oxidauth should support.

### Entities Involved
- **Oxidauth**: The Oxidauth instance used by a
- **Client**: The code base using Oxidauth for authentication
- **End User**: The client's user base, individuals
- **Identity Provider**: The entity providing the oauth (gsuite organization, azure organization, often a company)
- **Identify Platform**: The overarching platform the Identity Provider uses (Google, Microsoft, etc..)
** Sometimes the Identity Provider is the Platform, like if Facebook is used directly

### Technical Flow
The following steps outline the oauth flow. Each step is addressed individually in greater detail below.
0. an end user initiates the oauth process from the client website
1. Client will reach out to oxidauth at `/auth/sso/oauth/build_redirect` to receive a redirect url to the identity platform
2. Oxidauth constructs the redirect url from a combination of values provided by Client and stored in Oxidauth
3. Oxidauth returns the redirect url to the Client
4. Client sends the end user to redirect url
5. End user authenticates (or fails to authenticate) with the identity provider
6. (If authentication failure) user is sent back to the login page
7. (If authentication success) identity platform sends end user to the redirect uri (a client server side address) and provides a token
8. Client calls oxidauth at `/auth/sso/oauth/exchange_token` and provides the token
9. Oxidauth uses the token and other values to post a request to identity platform to exchange the token for an access code
10. Oxidauth uses the resulting access_code and other values to post a request to identity platform for the end user's information
11. Oxidauth checks end user profile for scope consent and authenetication status
12. Oxidauth sends user profile information back to client
13. Client uses the profile information to sign in, sign up, or otherwise handle the user account next steps

#### 0. Initiation
Example client oauth initiation screen:
<img width="743" alt="Screenshot 2024-04-12-login" src="./images/oauth-initiation.png">

#### 1. Redirect Url
The oath redirect url is constructed from many values, which are held between the client and oxidauth, many of which must match the values stated by the identity provider in their oauth setup.

##### Redirect Url ingredients:
- client id (source: stored in oxidauth DB table: `oauth_secrets`)
- scopes (source: stored in oxidauth DB table: `oauth_secrets`)
- redirect_uri (source: provided by client to oxidauth)
- identity platform oauth base url (source: oxidauth authority settings?)
- hd - this is an optional email domain specifier. Identity Platform will only show account options with this domain. (source: provided by client)

#### 2. Constructed redirect url
Ex Redirect Url: "https://accounts.google.com/o/oauth2/v2/auth?{redirect_uri}{client_id}{scopes}{hd}response_type=code&include_granted_scopes=true",

#### 5. Identity Platform Authentication


#### 8. Exchange Token
The token provided by the identity platform is combined with other values and posted back to the identity platform to get an access token that is needed in order to request user info. There are two separate calls to the identity platform to go from token to user profile.

##### Exchange Token ingredients:
- Header: "application/x-www-form-urlencoded" - request is form type
- token (source: provided by client who just got it from identity platform),
- client_id (source: stored in oxidauth DB table: `oauth_secrets`)
- client_secret (source: stored in oxidauth DB table: `oauth_secrets`)
- redirect_uri (source: provided by client to oxidauth)
- grant_type - static "authorization code"
- exchange_endpoint - identity platform token exchange url, does not change (source: oxidauth authority settings)

Exchange token request (if successful) returns an access_token, which is passed as the bearer token in the following request.

##### Profile Information Request ingredients
- Header: AUTHORIZATION, access token as bearer
- profile information request endpoint - identity platform url (which is specific to the scopes requested.. common sense source?)

Returns the user information contained in the scopes requested.

### New Endpoints
- [POST] /auth/sso/oauth/build_redirect - takes in parameters from the client, constructs a redirect url from provided and stored values, and returns a url
- [POST] /auth/sso/oauth/exchange_token - takes in oauth result from the client, exchanges provided token for user information from the identity platform, and returns the user information

### Other changes
- OxidAuth Authority will now have an added property of `require-2fa`, which, if true, will create a 2fa code for each new user and impact the sign in flow.
- The authenticate logic will now have two paths. If the username password authority does not require 2FA, the flow remains unchanged and upon username password validation, returns the full JWT. We are adding a branch in this logic to detect if 2FA is required, and instead trigger the email send and return a limited JWT with only the permissions to see the email code page.

### Libraries
Code generation library: [Boring Auth](https://docs.rs/boringauth) This package is a library designed to provide out-of-box HOTP and TOTP clients to generate one-time passwords.

### Database Migrations
Table: oauth_secrets
Columns: id, client_id, oauth_id, oauth_secret, created_at

### Research & Resources
- Google Oauth2 implementation guide: https://developers.google.com/identity/protocols/oauth2/web-server#handlingresponse

THOUGHTS
- if we store scopes and scope_url here... oauth secrets DB table name kind of sucks because it holds more than the secrets...
** Sometimes the Identity Provider is the Platform, like if Facebook is used directly... would this even work?
- should /auth/sso/oauth/exchange_token return the user profile? .. it can't create a user, so it has to only return the user info for the client to decide what to do with
- worth it to throw static values in the config or oauth secrets in case they later become... not static?
- because the profile information request goes to the endpoint specific to the scope being requested.. it might make sense for the client to supply both of these together?
