use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use oxidauth_kernel::error::BoxedError;
use serde::Serialize;

use super::{APPLE_ISSUER, AppleAuthError};
use crate::auth::strategies::oauth2::AuthorityParams;

/// Apple caps client-secret lifetime at 6 months; a 5-minute expiry shrinks
/// the blast radius of a leaked JWT to roughly nothing (OXA-000072).
pub const CLIENT_SECRET_TTL_SECONDS: u64 = 300;

#[derive(Debug, Serialize)]
struct ClientSecretClaims {
    iss: String, // Apple Team ID
    aud: String, // https://appleid.apple.com — Apple requires a STRING aud
    sub: String, // Services ID (= the flow's client_id / `oauth2_id`)
    iat: u64,
    exp: u64,
}

/// Signs the per-request client-secret JWT Apple demands in place of a static
/// client secret: ES256 over `iss`/`aud`/`sub`/`iat`/`exp` with the `kid`
/// header naming the Services key. The signing key is read from the env var
/// *named* by the authority row — private key bytes never live in the DB
/// (same indirection as the username_password pepper, OXA-000072).
#[tracing::instrument(name = "apple client secret sign", skip(params))]
pub fn build_client_secret(params: &AuthorityParams) -> Result<String, BoxedError> {
    let team_id = params
        .apple_team_id
        .as_deref()
        .ok_or(AppleAuthError::MissingParam("apple_team_id"))?;
    let key_id = params
        .apple_key_id
        .as_deref()
        .ok_or(AppleAuthError::MissingParam("apple_key_id"))?;
    let env_var = params
        .apple_private_key_env_var
        .as_deref()
        .ok_or(AppleAuthError::MissingParam("apple_private_key_env_var"))?;

    let pem =
        std::env::var(env_var).map_err(|_| AppleAuthError::MissingEnvVar(env_var.to_owned()))?;

    let key =
        EncodingKey::from_ec_pem(pem.as_bytes()).map_err(|_| AppleAuthError::InvalidPrivateKey)?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AppleAuthError::Signing)?
        .as_secs();

    let mut header = Header::new(Algorithm::ES256);
    header.kid = Some(key_id.to_owned());

    let claims = ClientSecretClaims {
        iss: team_id.to_owned(),
        aud: APPLE_ISSUER.to_owned(),
        sub: params.oauth2_id.clone(),
        iat: now,
        exp: now + CLIENT_SECRET_TTL_SECONDS,
    };

    encode(&header, &claims, &key).map_err(|_| AppleAuthError::Signing.into())
}

#[cfg(test)]
mod tests {
    use jsonwebtoken::{DecodingKey, decode, decode_header};
    use oxidauth_kernel::authorities::AuthorityStrategy;

    use super::*;
    use crate::{
        EnvGuard,
        auth::strategies::oauth2::{
            apple::fixtures::{EC_PEM, EC_PUB_PEM},
            fixtures::{APPLE_KEY_ENV_VAR, APPLE_KEY_ID, APPLE_TEAM_ID, apple_authority_params},
        },
    };

    #[derive(serde::Deserialize)]
    struct Claims {
        iss: String,
        aud: String,
        sub: String,
        iat: u64,
        exp: u64,
    }

    #[test]
    fn client_secret_is_an_es256_jwt_with_apples_claims() {
        let _env = EnvGuard::set(APPLE_KEY_ENV_VAR, EC_PEM);
        let params = apple_authority_params();

        let jwt = build_client_secret(&params).expect("signs with the fixture key");

        let header = decode_header(&jwt).expect("decodable header");
        assert_eq!(header.alg, Algorithm::ES256);
        assert_eq!(
            header.kid.as_deref(),
            Some(APPLE_KEY_ID),
            "the kid header must name the Services key"
        );

        let mut validation = jsonwebtoken::Validation::new(Algorithm::ES256);
        validation.set_audience(&[APPLE_ISSUER]);

        let data = decode::<Claims>(
            &jwt,
            &DecodingKey::from_ec_pem(EC_PUB_PEM.as_bytes()).expect("fixture pub key"),
            &validation,
        )
        .expect("signature verifies against the paired public key");

        assert_eq!(data.claims.iss, APPLE_TEAM_ID, "iss is the Team ID");
        assert_eq!(data.claims.aud, APPLE_ISSUER, "aud is apple");
        assert_eq!(
            data.claims.sub, "sso-client-id",
            "sub is the Services ID (oauth2_id)"
        );

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_secs();
        assert!(
            (data.claims.iat..=data.claims.exp).contains(&now)
                && data.claims.exp - data.claims.iat == CLIENT_SECRET_TTL_SECONDS,
            "the secret must be live now and expire on the fixed short TTL"
        );
    }

    #[test]
    fn misconfiguration_fails_typed_never_panics() {
        let params = apple_authority_params();

        // each EnvGuard acquires the process-wide ENV_LOCK, so they must be
        // scoped one-at-a-time (nested guards would self-deadlock)
        {
            // env var named by the fixture is absent unless a test binds it
            let _env = EnvGuard::unset(APPLE_KEY_ENV_VAR);
            let err = build_client_secret(&params).expect_err("missing env var must error");
            assert!(
                err.to_string()
                    .contains(APPLE_KEY_ENV_VAR)
                    && err
                        .to_string()
                        .contains("is not set"),
                "got: {err}"
            );
        }

        {
            // PEM present but not a key
            let _env = EnvGuard::set(APPLE_KEY_ENV_VAR, "not a key");
            let err = build_client_secret(&params).expect_err("garbage PEM must error");
            assert!(
                err.to_string()
                    .contains("PKCS#8"),
                "got: {err}"
            );
        }

        // flavor params absent entirely
        let mut bare = crate::auth::strategies::oauth2::fixtures::authority_params(
            crate::auth::strategies::oauth2::OAuthFlavors::Apple,
        );
        bare.apple_team_id = None;
        let err = build_client_secret(&bare).expect_err("missing team id must error");
        assert!(
            err.to_string()
                .contains("apple_team_id"),
            "got: {err}"
        );
    }

    #[test]
    fn apple_flavor_is_a_distinct_strategy_provider_shape() {
        // the flavor is pure authority-JSONB config: the strategy stays
        // `oauth2`, the DB CHECK (`provider LIKE 'oauth2%'`) is already
        // satisfied, and no migration is needed for apple rows
        assert_eq!(AuthorityStrategy::Oauth2.to_string(), "oauth2");

        let params = apple_authority_params();
        let json = serde_json::to_value(&params).expect("serializes");
        assert_eq!(json["flavor"], "Apple");
        assert_eq!(json["apple_team_id"], APPLE_TEAM_ID);
        assert_eq!(json["apple_private_key_env_var"], APPLE_KEY_ENV_VAR);
        assert!(
            json.get("apple_key_id")
                .is_some(),
            "kid must round-trip through the authority params"
        );
    }

    #[test]
    fn google_style_rows_deserialize_without_the_apple_fields() {
        // additive serde optionals: a stored Google row (no apple_*) loads
        // unchanged — this is the backward-compat gate for the new fields
        let legacy = serde_json::json!({
            "exchange_url": "https://oauth2.example.com/token",
            "flavor": "Google",
            "oauth2_id": "sso-client-id",
            "oauth2_secret": "sso-client-secret",
            "profile_url": "https://profile.example.com/me",
            "scopes": "openid email profile",
            "redirect_url": "https://authorize.example.com/oauth",
            "client_base_url": "https://app.example.com",
            "redirect_uri": "https://app.example.com/cb",
        });

        let params: crate::auth::strategies::oauth2::AuthorityParams =
            serde_json::from_value(legacy).expect("legacy row loads");
        assert!(params.apple_team_id.is_none());
        assert!(params.apple_key_id.is_none());
        assert!(
            params
                .apple_private_key_env_var
                .is_none()
        );
    }
}
