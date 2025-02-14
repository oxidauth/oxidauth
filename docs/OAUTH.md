# Oauth in Oxidauth

### Entities Involved
- **Oxidauth**: The Oxidauth instance
- **Client**: The code base using Oxidauth for authentication
- **End User**: The client's user base, individuals
- **Identity Provider**: The entity providing the oauth (gsuite organization, azure organization, often a company)
- **Identify Platform**: The overarching platform the Identity Provider uses (Google, Microsoft, etc..)
** Sometimes the Identity Provider is the Platform, like if Facebook is used directly

### Technical Flow
The following steps outline the oauth flow. Each step is addressed individually in greater detail below.
1. An end user initiates the oauth process from the client website (clicks oauth sign up button)
2. Client will reach out to oxidauth at `/auth/sso/oauth/build_redirect` to receive a redirect url to the identity platform
3. Oxidauth constructs the redirect url from a combination of values provided by Client and stored in Oxidauth
4. Oxidauth returns the redirect url to the Client
5. Client sends the end user to redirect url
6. End user authenticates (or fails to authenticate) with the identity provider
7. (If authentication failure) user is sent back to the login page
8. (If authentication success) identity platform sends end user to the redirect uri (a client server side address) and provides a token
9. Client calls oxidauth at `/auth/sso/oauth/exchange_token` and provides the token
10. Oxidauth uses the token and other values to post a request to identity platform to exchange the token for an access code
11. Oxidauth uses the resulting access_code and other values to post a request to identity platform for the end user's information
12. Oxidauth checks end user profile for scope consent and authenetication status
13. Oxidauth sends user profile information back to client
14. Client uses the profile information to sign in, sign up, or otherwise handle the user account next steps

### Steps Breakdown

#### 1. Initiation
Example client oauth initiation screen:

<img width="743" alt="Screenshot 2024-04-12-login" src="./images/oauth-initiation.png">

#### 2. Redirect Url
The oath redirect url is constructed from many values which are held between the client and oxidauth, many of which must match the values stated by the identity provider in their oauth setup.

##### Redirect Url ingredients:
- client_id (source: stored in oxidauth DB table: `oauth_secrets`)
- scopes (source: stored in oxidauth DB table: `oauth_secrets`)
- redirect_uri (source: provided by client to oxidauth)
- identity platform oauth base url (source: oxidauth authority settings?)
- hd - this is an optional email domain specifier. Identity Platform will only show account options with this domain. (source: provided by client)

#### 3. Constructed redirect url
Ex Redirect Url: "https://accounts.google.com/o/oauth2/v2/auth?{redirect_uri}{client_id}{scopes}{hd}response_type=code&include_granted_scopes=true",

#### 6. Identity Platform Authentication
Example identity platform authentication screen:

#### 9. Exchange Token
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

### Implementation Examples
#### Scenario #1
Every identity platform (but not every identity provider) has their own authority. Here is an example scenario:

ExampleCo uses Oxidauth for authentication. They use gsuite for their employee accounts and want to provide authentication through google sso.
ExampleCo would need the following authority in their authorities table:
- ExampleCo Web Google Oauth - auth strategy: oauth

Now ExampleCo is using oxidauth, so they are the client. But it also has ownership of the google account(& email domain) offering the authentication, so it is the identity provider. Google is the identity platform.

By controlling the settings in their google console, the UI in ExampleCo Web, and the settings in Oxidauth, ExampleCo can walk through all the SSO steps outlined above.

#### Scenario #2
Now lets look at a slightly more complex example. Lets say ExampleCo develops a web platform as a product and sells access to that platform to a customer company (lets call that company CatsInc).
That customer company wants to give their employees access to ExampleCo's great product, but they want their employees to authenticate with sso through their company microsoft account.
In this scenario, ExampleCo is the client, CatsInc is the Identity Provider, and Microsoft is the Identity Platform.

Now ExampleCo has multiple Authorities providing oauth to the Web app:
- ExampleCo Web Google Oauth - auth strategy: oauth
- ExampleCo Web Microsoft Oauth - auth strategy: oath

Both authorities use the oauth strategy, but require different settings, so they should be separate entries.

Now ExampleCo doesn't have access to the identity provider settings, so in order to set up oxidauth correctly they have to exchange some information with CatsInc. Here's what needs to be exchanged:
CatsInc provides:
- which identity platform, and potentially version/flavor of auth
- their sso client id
- their sso client secret

ExampleCo provides:
- the redirect_uri

If any of the information above changes on either side at any time, the whole oauth process will fail and users will not be able to sign in.
Further, both companies need to be clear on what user data points are required by ExampleCo for their app to function, and where (what scopes) those data points live in on the CatsInc side.

With the information above, ExampleCo should be able to set up their UI in ExampleCo Web and the settings in Oxidauth to set up a successful oauth2 flow.

### Resources
- Google Oauth2 implementation guide: https://developers.google.com/identity/protocols/oauth2/web-server
- Google Oauth2 Best Practices: https://developers.google.com/identity/protocols/oauth2/resources/best-practices
- Oauth2 spec: https://datatracker.ietf.org/doc/html/rfc6749
- The oauth website: https://oauth.net
