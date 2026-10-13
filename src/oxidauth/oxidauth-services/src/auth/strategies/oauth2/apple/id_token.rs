use base64::{Engine as _, prelude::BASE64_URL_SAFE_NO_PAD};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use oxidauth_kernel::{auth::authenticate_or_register::OAuth2Profile, error::BoxedError};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use url::Url;

use super::{APPLE_ISSUER, AppleAuthError};
use crate::auth::strategies::oauth2::AuthorityParams;

#[derive(Debug, Clone, Deserialize)]
pub struct AppleJwk {
    pub kid: String,
    pub n: String,
    pub e: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppleJwks {
    pub keys: Vec<AppleJwk>,
}

/// The id_token claims this flow depends on. `email_verified` arrives as a
/// bool from Apple but OIDC allows the `"true"` string, so it is kept loose.
#[derive(Debug, Clone, Deserialize)]
pub struct AppleIdTokenClaims {
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub nonce: Option<String>,
    #[serde(default)]
    pub email_verified: Option<serde_json::Value>,
}

/// Fetches Apple's signing keys. The authority's `profile_url` stores the
/// JWKS endpoint (`https://appleid.apple.com/auth/keys`) — Apple has no
/// userinfo endpoint, so the slot that means "where identity comes from"
/// for the other flavors means "where identity is verified" here.
#[tracing::instrument(name = "apple jwks fetch")]
pub async fn fetch_apple_jwks(url: &Url) -> Result<AppleJwks, BoxedError> {
    let response = reqwest::Client::new()
        .get(url.clone())
        .send()
        .await
        .map_err(|_| AppleAuthError::Transport)?;

    let status = response.status();

    if !status.is_success() {
        tracing::warn!(
            status = status.as_u16(),
            "apple jwks endpoint returned a non-success status",
        );

        return Err(AppleAuthError::IdpRejected {
            status: status.as_u16(),
            error: None,
        }
        .into());
    }

    Ok(response
        .json()
        .await
        .map_err(|_| AppleAuthError::TokenRejected("jwks document unparseable".to_owned()))?)
}

/// Verifies an Apple id_token against a JWKS document and the consumed state
/// nonce — the OIDC job the Google/Microsoft paths never do (their id_tokens
/// are discarded; see M4/OXA-000037). Checks: RS256 signature by the `kid`
/// Apple named, `iss`/`aud`/`exp`, and `nonce == base64url(sha256(state))`,
/// which binds this token to a single-use, server-minted nonce the callback
/// already consumed (OXA-000010 doing its actual anti-replay job).
pub fn verify_apple_id_token(
    id_token: &str,
    state: &str,
    jwks: &AppleJwks,
    services_id: &str,
) -> Result<OAuth2Profile, BoxedError> {
    let header = decode_header(id_token)
        .map_err(|err| AppleAuthError::TokenRejected(format!("{:?}", err.kind())))?;

    let kid = header
        .kid
        .ok_or_else(|| AppleAuthError::TokenRejected("no kid header".to_owned()))?;

    let jwk = jwks
        .keys
        .iter()
        .find(|key| key.kid == kid)
        .ok_or_else(|| AppleAuthError::TokenRejected("kid not in apple jwks".to_owned()))?;

    let key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
        .map_err(|err| AppleAuthError::TokenRejected(format!("{:?}", err.kind())))?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[APPLE_ISSUER]);
    validation.set_audience(&[services_id]);

    let data = decode::<AppleIdTokenClaims>(id_token, &key, &validation)
        .map_err(|err| AppleAuthError::TokenRejected(format!("{:?}", err.kind())))?;

    let expected_nonce = BASE64_URL_SAFE_NO_PAD.encode(Sha256::digest(state.as_bytes()));

    if data.claims.nonce.as_deref() != Some(expected_nonce.as_str()) {
        return Err(AppleAuthError::NonceMismatch.into());
    }

    let verified_ok = match &data.claims.email_verified {
        None => true,
        Some(serde_json::Value::Bool(v)) => *v,
        Some(serde_json::Value::String(v)) => v == "true",
        Some(_) => false,
    };

    match (data.claims.email, verified_ok) {
        (Some(email), true) if !email.is_empty() => {
            Ok(OAuth2Profile {
                email,
                // OXA-000072 decision: no display names — Apple delivers them
                // exactly once (first consent) and they can never be re-fetched,
                // so the flow refuses to depend on them.
                given_name: None,
                family_name: None,
            })
        },
        _ => Err(AppleAuthError::UnverifiedEmail.into()),
    }
}

/// Apple's "retrieve profile" — verify the id_token from the code exchange
/// and read the identity out of it.
#[tracing::instrument(name = "apple oauth retrieve profile")]
pub async fn retrieve_apple_profile(
    id_token: String,
    state: &str,
    params: &AuthorityParams,
) -> Result<OAuth2Profile, BoxedError> {
    let jwks = fetch_apple_jwks(&params.profile_url).await?;

    verify_apple_id_token(&id_token, state, &jwks, &params.oauth2_id)
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use jsonwebtoken::{EncodingKey, Header, encode};

    use super::*;
    use crate::auth::strategies::oauth2::apple::fixtures::{
        IdTokenClaims,
        KID,
        RSA_PEM,
        jwks_json,
        nonce_for,
        sign_id_token as sign,
    };

    fn jwks() -> AppleJwks {
        serde_json::from_value(jwks_json()).expect("fixture jwks")
    }

    fn claims(state: Option<&str>) -> IdTokenClaims {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_secs();

        IdTokenClaims {
            iss: APPLE_ISSUER.to_owned(),
            aud: "sso-client-id".to_owned(),
            exp: now + 300,
            iat: now,
            sub: "001487.8f718d0da2a0432994da98a1e41151cb.1447".to_owned(),
            email: Some("user@hide.myapplemail.com".to_owned()),
            nonce: state.map(nonce_for),
            email_verified: Some(serde_json::Value::Bool(true)),
        }
    }

    fn verify(id_token: &str, state: &str) -> Result<OAuth2Profile, BoxedError> {
        verify_apple_id_token(id_token, state, &jwks(), "sso-client-id")
    }

    #[test]
    fn a_correctly_signed_token_yields_the_email() {
        let state = "kkdLGJ3Z8cZ0X9hL3E0R4g";

        let profile = verify(&sign(claims(Some(state))), state).expect("valid token verifies");

        assert_eq!(profile.email, "user@hide.myapplemail.com");
        assert!(
            profile.given_name.is_none() && profile.family_name.is_none(),
            "apple profiles carry no display names (OXA-000072)"
        );
    }

    #[test]
    fn a_token_bound_to_a_different_state_is_rejected() {
        // replay shape: a genuine Apple token captured from a DIFFERENT
        // login attempt must not authenticate this one
        let err = verify(
            &sign(claims(Some("kkdLGJ3Z8cZ0X9hL3E0R4g"))),
            "sQ3mnbF1zX8V2wK6pL0YdA",
        )
        .expect_err("nonce must bind the token to the consumed state");

        assert!(
            err.to_string()
                .contains("nonce"),
            "got: {err}"
        );
    }

    #[test]
    fn forged_keys_and_wrong_recipients_are_rejected() {
        let state = "kkdLGJ3Z8cZ0X9hL3E0R4g";

        // token signed with a key NOT in Apple's JWKS (unknown kid)
        let mut foreign = Header::new(Algorithm::RS256);
        foreign.kid = Some("someoneElsesKey".to_owned());
        let token = encode(
            &foreign,
            &claims(Some(state)),
            &EncodingKey::from_rsa_pem(RSA_PEM.as_bytes()).expect("fixture key"),
        )
        .expect("signs");
        let err = verify(&token, state).expect_err("kid lookup must fail");
        assert!(
            err.to_string()
                .contains("kid"),
            "got: {err}"
        );

        // right key, wrong audience (token minted for another Services ID)
        let mut wrong_aud = claims(Some(state));
        wrong_aud.aud = "not-our-services-id.app".to_owned();
        let err = verify(&sign(wrong_aud), state).expect_err("aud must be checked");
        assert!(
            err.to_string()
                .contains("InvalidAudience"),
            "got: {err}"
        );

        // right everything, expired
        let mut expired = claims(Some(state));
        expired.exp = 1;
        expired.iat = 0;
        let err = verify(&sign(expired), state).expect_err("exp must be checked");
        assert!(
            err.to_string()
                .contains("ExpiredSignature"),
            "got: {err}"
        );
    }

    #[test]
    fn unverified_or_missing_emails_are_rejected() {
        let state = "kkdLGJ3Z8cZ0X9hL3E0R4g";

        let mut unverified = claims(Some(state));
        unverified.email_verified = Some(serde_json::Value::Bool(false));
        let err = verify(&sign(unverified), state).expect_err("email_verified=false must fail");
        assert!(
            err.to_string()
                .contains("email"),
            "got: {err}"
        );

        let mut missing_email = claims(Some(state));
        missing_email.email = None;
        verify(&sign(missing_email), state).expect_err("no email must fail");

        // the OIDC string form of verified ("true") is accepted
        let mut stringy = claims(Some(state));
        stringy.email_verified = Some(serde_json::Value::String("true".to_owned()));
        verify(&sign(stringy), state).expect("string-true verifies");
    }

    #[test]
    fn jwks_document_shape_tolerates_unknown_fields() {
        // the fixture above carries kty/use/alg alongside kid/n/e — only the
        // three fields we need are required to deserialize
        assert_eq!(jwks().keys.len(), 1);
        assert_eq!(jwks().keys[0].kid, KID);
    }
}
