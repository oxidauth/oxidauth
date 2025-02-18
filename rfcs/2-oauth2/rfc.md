# Summary

Add Oauth2 (Google flavor only) as an authentication strategy supported by Oxidauth.

### Benefit

Oauth2 is a popular authentication standard that Oxidauth should support. This RFC provides the underlying strucuture to support Oauth2 as well as the implementation details for the Google Oauth2 flow. The intention is that other platforms can be added using this structure.

### Entities Involved
- **Oxidauth**: The Oxidauth instance
- **Client**: The code base using Oxidauth for authentication
- **End User**: The client's user base, individuals
- **Identity Provider**: The entity providing the oauth (gsuite organization, azure organization, often a company) - each of these will have its own row in the authorities table
- **Oauth2 Platform**: The overarching platform the Identity Provider uses (Google, Microsoft, etc..)
** Sometimes the Identity Provider is the Platform, like if Facebook is used directly

### New Endpoints
- [POST] /auth/oauth2/redirect - takes in parameters from the client, constructs a redirect url from provided and stored values, and returns a url
- [POST] /auth/oauth2/authenticate - takes in oauth result from the client, exchanges provided token for user information from the identity platform, and returns the user information

### Code changes
- Addition of new auth strategy: oauth2
- Addition of new authority params type:
```

```

### Database Migrations
None

### Technical Flow
The following steps outline the oauth flow. Each step is addressed individually in greater detail below.
1. An end user initiates the oauth process from the client website (clicks oauth sign up button)
2. Client will reach out to oxidauth at `/auth/oauth2/redirect` to receive a redirect url to the identity platform
3. Oxidauth constructs the redirect url from values stored in Oxidauth
4. Oxidauth returns the redirect url to the Client
5. Client sends the end user to redirect url
6. End user authenticates (or fails to authenticate) with the identity provider
7. (If authentication failure) user is sent back to the login page
8. (If authentication success) identity platform sends end user to the redirect uri (`/auth/oauth2/authenticate/:organization_id`) and provides a token
9. Oxidauth uses the token and other values to post a request to identity platform to exchange the token for an access code
10. Oxidauth uses the resulting access_code and other values to post a request to identity platform for the end user's information
11. Oxidauth checks end user profile for scope consent and authenetication status
12. Oxidauth sends user profile information back to client
13. Client uses the profile information to sign in, sign up, or otherwise handle the user account next steps

#### 1. Initiation
Example client oauth initiation screen:

<img width="743" alt="Screenshot 2024-04-12-login" src="./images/oauth-initiation.png">

#### 2. Redirect Url
The oath2 redirect url is constructed from multiple values, all stored within oxidauth authority settings.

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

### Implementation
This setup means that every identity provider has their own authority row. Settings on the particular oauth2 implementation can be managed via the authority params (like if a company changes its oauth id or updates its oauth secret).


### Research & Resources
- Google Oauth2 implementation guide: https://developers.google.com/identity/protocols/oauth2/web-server#handlingresponse
