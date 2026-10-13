pub mod exchange_token;
pub mod retrieve_profile;

use std::{error::Error, fmt};

pub use exchange_token::exchange_google_token;
use oxidauth_kernel::{
    JsonValue,
    auth::authenticate_or_register::OAuth2Profile,
    error::BoxedError,
};
pub use retrieve_profile::retrieve_google_profile;
use serde::{Deserialize, Serialize};

/// Raised when Google itself flags the account email as unverified
/// (OXA-000073). Sanitized display — the address is the user's own, but
/// the error stays free of payload prose, in the `OAuth2StateError` /
/// `AppleAuthError` family.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoogleProfileError {
    EmailUnverified,
}

impl fmt::Display for GoogleProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use GoogleProfileError::*;

        match self {
            EmailUnverified => write!(f, "google account email is not verified"),
        }
    }
}

impl Error for GoogleProfileError {
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleExchangeTokenReq {
    pub code: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub grant_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleExchangeTokenRes {
    pub access_token: String,
    pub expires_in: u32,
    pub scope: String,
    pub token_type: String,
    pub id_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleProfile {
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub id: Option<String>,
    pub email: String,
    pub verified_email: bool,
}

impl TryFrom<JsonValue> for GoogleProfile {
    type Error = BoxedError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        let profile = serde_json::from_value(value.inner_value())?;

        Ok(profile)
    }
}

impl GoogleProfile {
    /// Maps the userinfo payload onto the flow's profile, refusing addresses
    /// Google itself flags unverified (OXA-000072 did for Apple). The parse
    /// boundary (`TryFrom<JsonValue>`) stays wire-shape-only; this is where
    /// the flag stops being decorative — before any authenticate/register
    /// hop can create or touch a `user_authorities` row.
    pub fn into_oauth_profile(self) -> Result<OAuth2Profile, GoogleProfileError> {
        if !self.verified_email {
            return Err(GoogleProfileError::EmailUnverified);
        }

        Ok(OAuth2Profile {
            email: self.email,
            given_name: self.given_name,
            family_name: self.family_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::auth::strategies::oauth2::{OAuthFlavors, fixtures::authority_params};

    #[test]
    fn exchange_token_request_wire_contract_from_authority_params() {
        // mirrors the field mapping inside `exchange_google_token`: the form
        // body posted to `exchange_url` must carry exactly these wire names
        let params = authority_params(OAuthFlavors::Google);
        let req = GoogleExchangeTokenReq {
            code: "one-time-code".to_owned(),
            client_id: params.oauth2_id.clone(),
            client_secret: params.oauth2_secret.clone(),
            redirect_uri: params
                .redirect_uri
                .to_string(),
            grant_type: "authorization_code".to_owned(),
        };

        let wire: Value = serde_json::to_value(&req).expect("serializable");

        assert_eq!(
            wire,
            json!({
                "code": "one-time-code",
                "client_id": "sso-client-id",
                "client_secret": "sso-client-secret",
                "redirect_uri": "https://app.example.com/cb",
                "grant_type": "authorization_code",
            }),
        );
    }

    #[test]
    fn exchange_token_response_requires_every_google_field() {
        let res: GoogleExchangeTokenRes = serde_json::from_value(json!({
            "access_token": "ya29.access-token",
            "expires_in": 3599,
            "scope": "openid email profile",
            "token_type": "Bearer",
            "id_token": "eyJhbGciOi.jwt",
        }))
        .expect("a well-formed google token response parses");

        assert_eq!(res.access_token, "ya29.access-token");

        // pinned: every field is required — an OAuth *error* response
        // ({"error": "invalid_grant"}) fails as a serde parse error, not as a
        // mapped auth error, and even a success response missing `id_token`
        // aborts the exchange
        assert!(
            serde_json::from_value::<GoogleExchangeTokenRes>(json!({
                "access_token": "ya29.access-token",
                "expires_in": 3599,
                "scope": "openid email profile",
                "token_type": "Bearer",
            }))
            .is_err()
        );
        assert!(
            serde_json::from_value::<GoogleExchangeTokenRes>(json!({
                "error": "invalid_grant",
            }))
            .is_err()
        );
    }

    #[test]
    fn profile_try_from_maps_optional_fields_and_requires_email() {
        let profile: GoogleProfile = GoogleProfile::try_from(JsonValue::new(json!({
            "name": null,
            "given_name": "Ann",
            "family_name": "Lee",
            "picture": null,
            "id": "1234567890",
            "email": "ann@example.com",
            "verified_email": true,
        })))
        .expect("a userinfo response parses");

        assert_eq!(profile.email, "ann@example.com");
        assert_eq!(profile.given_name.as_deref(), Some("Ann"));
        assert_eq!(profile.family_name.as_deref(), Some("Lee"));
        assert!(profile.name.is_none());

        // OXA-000073 (flipped pin): the TRY boundary stays wire-shape-only,
        // but `false` no longer passes the profile boundary itself — see
        // `unverified_emails_are_refused_at_the_profile_boundary`
        let unverified: GoogleProfile = GoogleProfile::try_from(JsonValue::new(json!({
            "email": "ghost@example.com",
            "verified_email": false,
        })))
        .expect("the flag is deserialized at the parse boundary");
        assert!(!unverified.verified_email);

        assert!(
            GoogleProfile::try_from(JsonValue::new(json!({
                "email": "no-flags@example.com",
            })))
            .is_err(),
            "`verified_email` stays required on the wire (presence pins unchanged)"
        );
        assert!(
            GoogleProfile::try_from(JsonValue::new(json!({
                "verified_email": true,
            })))
            .is_err()
        );
    }

    #[test]
    fn unverified_emails_are_refused_at_the_profile_boundary() {
        // OXA-000073: the flag is no longer decorative — an address Google
        // itself flags unverified never reaches the authenticate/register
        // hops as a profile (Apple got the same bar from day one in
        // OXA-000072)
        let err = GoogleProfile::try_from(JsonValue::new(json!({
            "email": "ghost@example.com",
            "verified_email": false,
        })))
        .expect("parses")
        .into_oauth_profile()
        .expect_err("unverified email must be refused");

        assert!(
            matches!(err, GoogleProfileError::EmailUnverified),
            "expected the typed error, got: {err:?}"
        );
        assert!(
            err.to_string()
                .contains("not verified"),
            "got: {err}"
        );

        let profile = GoogleProfile::try_from(JsonValue::new(json!({
            "email": "ann@example.com",
            "verified_email": true,
            "given_name": "Ann",
            "family_name": "Lee",
        })))
        .expect("parses")
        .into_oauth_profile()
        .expect("verified email maps through");

        assert_eq!(profile.email, "ann@example.com");
        assert_eq!(profile.given_name.as_deref(), Some("Ann"));
        assert_eq!(profile.family_name.as_deref(), Some("Lee"));
    }
}
