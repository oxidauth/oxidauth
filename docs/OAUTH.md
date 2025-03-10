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
1. An end user initiates the oauth process from the client website (Ex: clicks oauth sign up button)
2. Client will reach out to oxidauth at `/auth/oauth2/redirect` to receive a redirect url to the identity platform
3. Oxidauth constructs the redirect url from values stored the authority params
4. Oxidauth returns the redirect url to the Client
5. Client sends the end user to redirect url
6. End user authenticates (or fails to authenticate) with the identity provider
7. (If authentication failure) user is sent back to the login page
8. (If authentication success) identity platform sends end user to the redirect uri, an oxidauth endpoint (`/auth/oauth2/authenticate/:authority_client_key`) and provides a token
9. Oxidauth uses the token and other values to post a request to identity platform to exchange the token for an access code
10. Oxidauth uses the resulting access_code and other values to post a request to identity platform for the end user's information
11. Oxidauth authenticates the user if they exist, and creates the user if they do not (Note: there is no difference between sign up and sign in with oauth)
12. Oxidauth redirects to the client (client address is an authority param) with a jwt and refresh token
13. Client uses the jwt and refresh token information to sign in, sign up, or otherwise handle the user account next steps

### Steps Breakdown

#### 1. Initiation
Example client oauth initiation screen:

<img width="743" alt="Screenshot 2024-04-12-login" src="./images/oauth-initiation.png">

#### 2. Redirect Url
The oath redirect url is constructed from many values which are held between the client and oxidauth, many of which must match the values stated by the identity provider in their oauth setup.

##### Redirect Url ingredients:
- client_id (identity provider sso id)
- scopes (for google, the areas of profile information to be received from the user's google profile)
- redirect_uri (identity provider redirect uri - an oxidauth address)
- identity platform oauth base url (built into the redirect url param in oxidauth)
- hd (optional google value to restrict which account options the user is presented with)

#### 3. Constructed redirect url
Ex Redirect Url: "https://accounts.google.com/o/oauth2/v2/auth?{redirect_uri}{client_id}{scopes}{hd}response_type=code&include_granted_scopes=true",

#### 6. Identity Platform Authentication
Example identity platform authentication screen:

#### 9. Exchange Token
The token provided by the identity platform is combined with other values and posted back to the identity platform to get an access token that is needed in order to request user info. There are two separate calls to the identity platform to go from token to user profile.

##### Exchange Token ingredients:
- Header "application/x-www-form-urlencoded" - request is form type
- token (source: query params on the incoming request. Google Ex: `auth/oauth2/authenticate/:organization_id?token=aksjds...&scopes=alslaf...`)
- client_id (source: authority params)
- client_secret (source: authority params)
- redirect_uri (source: authority params)
- grant_type - static "authorization code"
- exchange_endpoint - identity platform token exchange url, does not change (source: oxidauth authority params)

Exchange token request (if successful) returns an access_token, which is passed as the bearer token in the following request.

##### Profile Information Request ingredients
- Header AUTHORIZATION, access token as bearer
- profile information request endpoint - identity platform url (source: authority params)

Returns the user information contained in the scopes requested.

### Oauth2 Authority Setup
Example values for an authority set up to use oauth2 strategy:

- id (normal uuid)
- name (Ex: ExampleCo Google Oauth) ** it is best to put the platform in the name, so that you can add another platform later (Ex: ExampleCo Microsoft Oauth)
- client key (normal uuid)
- status: enabled
- strategy: oauth2
- settings: ```{"totp": "Disabled", "jwt_ttl": {"secs": 300, "nanos": 0}, "refresh_token_ttl": {"secs": 259200, "nanos": 0}, "entitlements_encoding": "txt"}```
- params: ```{
    "flavor": "Google",
    "scopes": "https://www.googleapis.com/auth/userinfo.profile https://www.googleapis.com/auth/userinfo.email",
    "oauth2_id": <your oauth2 id>,
    "oauth2_secret": <your oauth2 secret>,
    "profile_url": "https://www.googleapis.com/userinfo/v2/me",
    "exchange_url": "https://oauth2.googleapis.com/token",
    "redirect_uri": "http://localhost:8001/api/v1/auth/oauth2/callback/156562bc-dad7-4a8a-81f3-2c1ef80e9b29",
    "redirect_url": "https://accounts.google.com/o/oauth2/v2/auth?client_id=127751927363-4l0710vnomm37imtagelivu0sn8rui3b.apps.googleusercontent.com&redirect_uri=http://localhost:8001/api/v1/auth/oauth2/callback/156562bc-dad7-4a8a-81f3-2c1ef80e9b29&scope=https://www.googleapis.com/auth/userinfo.profile https://www.googleapis.com/auth/userinfo.email",
    "client_base_url": "http://app.mindly.localhost"
  }```

### Implementation Examples
#### Scenario #1
ExampleCo uses Oxidauth for authentication. They use gsuite for their employee accounts and want to provide authentication through google sso.
ExampleCo would need the following authority in their authorities table:
- ExampleCo Web Google Oauth - auth strategy: oauth2

Now ExampleCo is using oxidauth, so they are the client. But it also has ownership of the google account(& email domain) offering the authentication, so it is the identity provider too. Google is the identity platform. Every identity provider has their own row in the authorities table.

By controlling the settings in their google console, the UI in ExampleCo Web, and the settings in Oxidauth, ExampleCo can walk through all the SSO steps outlined above.

#### Scenario #2
Now lets look at a slightly more complex example. Lets say ExampleCo develops a web platform as a product and sells access to that platform to a customer company (lets call that company CatsInc).
CatsInc wants to give their employees access to ExampleCo's great product, but they want their employees to authenticate with sso through their own CatsInc google account.
In this scenario, ExampleCo is the client, CatsInc is the Identity Provider, and Google is the Identity Platform.

Now ExampleCo has two multiple Authorities providing Google oauth in the Web app:
- ExampleCo Web Google Oauth - auth strategy: oauth
- CatsInc Web Google Oauth - auth strategy: oath

Both authorities use the oauth strategy, but require different settings, so they should be separate entries. This way ExampleCo and CatsInc oauth settings can be easily managed independently from one place.

To set this up, ExampleCo doesn't have access to the identity provider settings and information so they have to exchange some information with CatsInc. Here's what needs to be exchanged:

CatsInc provides:
- which identity platform, and potentially version/flavor of auth (if its oauth1, or SAML the oauth2 setup instructions do not apply)
- their sso client id
- their sso client secret
- the user information they are able/willing to provide

ExampleCo provides:
- the oxidauth redirect_uri

If any of the information above changes on either side at any time, the whole oauth process will fail and users will not be able to sign in.
Further, both companies need to be clear on what user data points are required by ExampleCo for their app to function, and where (what scopes) those data points live in on the CatsInc side.

With the information above, ExampleCo should be able to set up their UI in ExampleCo Web and the settings in Oxidauth to create a successful oauth2 flow.

### Resources
- Google Oauth2 implementation guide: https://developers.google.com/identity/protocols/oauth2/web-server
- Google Oauth2 Best Practices: https://developers.google.com/identity/protocols/oauth2/resources/best-practices
- Oauth2 spec: https://datatracker.ietf.org/doc/html/rfc6749
- The oauth website: https://oauth.net
