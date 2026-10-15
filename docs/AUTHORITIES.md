# Authorities
Oxidauth Authorities are an instance of an authorization strategy and a central building block of the Oxidauth system.

## How it works
// todo

## Types of Authorites
// todo

## Relationships
// todo

### User Authorities
// todo

## Sign in with Apple (OXA-000072)

Apple is an OAuth2 authority (`strategy: oauth2`) whose params carry
`"flavor": "Apple"`. It differs from Google/Microsoft in three ways: there
is no static client secret (oxidauth signs a short-lived ES256 JWT per
request), identity arrives inside the token response's `id_token` (verified
against Apple's JWKS and bound to the single-use login state via
`nonce = sha256(state)`), and the callback returns as a form POST to
`POST /v1/auth/oauth2/callback/{client_key}`.

Authority `params` for an Apple row:

| param | value |
|---|---|
| `flavor` | `"Apple"` |
| `oauth2_id` | your **Services ID** (e.g. `rs.example.app`) |
| `oauth2_secret` | unused by the Apple flavor (any placeholder) |
| `exchange_url` | `https://appleid.apple.com/auth/token` |
| `profile_url` | `https://appleid.apple.com/auth/keys` (the JWKS slot *verifies* identity here) |
| `redirect_url` | `https://appleid.apple.com/auth/authorize?client_id=<Services ID>&scope=email&redirect_uri=<encoded callback>` |
| `redirect_uri` | `https://<host>/v1/auth/oauth2/callback/<client_key>` (must match Apple's configured return URL) |
| `client_base_url` | your frontend base URL |
| `scopes` | `email` (names are deliberately not requested — see below) |
| `apple_team_id` | your Apple **Team ID** (the client-secret JWT `iss`) |
| `apple_key_id` | the **Key ID** of the Sign in with Apple key (the JWT `kid`) |
| `apple_private_key_env_var` | **name** of the env var holding the `.p8` PEM — e.g. `OXIDAUTH_APPLE_PRIVATE_KEY_PEM` |

Setup in App Store Connect:

1. **Identifiers** → register a Services ID (`rs.example.app`); enable
   *Sign in with Apple*; set the return URL to the callback above.
2. **Keys** → create a key, enable *Sign in with Apple*, download the
   one-time `.p8` file.
3. Put the `.p8` contents in the named env var on the server (see
   `example.env`). The private key never enters the database: the authority
   row only names the variable, and exchange fails with a typed error if
   the variable is missing.
4. Create the authority with the params table above.

Display names are intentionally not stored: Apple delivers them exactly
once on first consent and they can never be re-fetched, so registration
proceeds on the verified email alone.
