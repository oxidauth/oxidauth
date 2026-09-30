pub mod exchange_token;
pub mod retrieve_profile;

pub use exchange_token::exchange_microsoft_token;
use oxidauth_kernel::{JsonValue, error::BoxedError};
pub use retrieve_profile::retrieve_microsoft_profile;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftExchangeTokenReq<'a> {
    pub client_id: &'a str,
    pub scope: &'a str,
    pub code: &'a str,
    pub redirect_uri: &'a str,
    pub client_secret: &'a str,
    pub grant_type: &'a str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftExchangeTokenRes {
    pub token_type: String,
    pub scope: String,
    pub expires_in: u32,
    pub ext_expires_in: u32,
    pub access_token: String,
    pub id_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftProfile {
    pub display_name: Option<String>,
    pub given_name: Option<String>,
    pub surname: Option<String>,
    pub id: String,
    pub mail: String,
}

impl TryFrom<JsonValue> for MicrosoftProfile {
    type Error = BoxedError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        let profile = serde_json::from_value(value.inner_value())?;

        Ok(profile)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::auth::strategies::oauth2::{OAuthFlavors, fixtures::authority_params};

    #[test]
    fn exchange_token_request_wire_contract_from_authority_params() {
        // mirrors the field mapping inside `exchange_microsoft_token`
        let params = authority_params(OAuthFlavors::Microsoft);
        let req = MicrosoftExchangeTokenReq {
            code: "one-time-code",
            scope: &params.scopes,
            client_id: &params.oauth2_id,
            client_secret: &params.oauth2_secret,
            redirect_uri: params.redirect_uri.as_ref(),
            grant_type: "authorization_code",
        };

        let wire: Value = serde_json::to_value(&req).expect("serializable");

        assert_eq!(
            wire,
            json!({
                "client_id": "sso-client-id",
                "scope": "openid email profile",
                "code": "one-time-code",
                "redirect_uri": "https://app.example.com/cb",
                "client_secret": "sso-client-secret",
                "grant_type": "authorization_code",
            }),
        );
    }

    #[test]
    fn exchange_token_response_requires_every_microsoft_field() {
        let res: MicrosoftExchangeTokenRes = serde_json::from_value(json!({
            "token_type": "Bearer",
            "scope": "openid email profile",
            "expires_in": 3599,
            "ext_expires_in": 3599,
            "access_token": "microsoft.access-token",
            "id_token": "eyJhbGciOi.jwt",
        }))
        .expect("a well-formed microsoft token response parses");

        assert_eq!(res.access_token, "microsoft.access-token");

        // pinned: all six fields required — responses without
        // `ext_expires_in` (Microsoft omits it in some tenants) fail to parse
        assert!(
            serde_json::from_value::<MicrosoftExchangeTokenRes>(json!({
                "token_type": "Bearer",
                "scope": "openid email profile",
                "expires_in": 3599,
                "access_token": "microsoft.access-token",
                "id_token": "eyJhbGciOi.jwt",
            }))
            .is_err()
        );
    }

    #[test]
    fn profile_uses_camel_case_wire_names_and_rejects_snake_case() {
        let profile: MicrosoftProfile = MicrosoftProfile::try_from(JsonValue::new(json!({
            "displayName": "Ann Lee",
            "givenName": "Ann",
            "surname": "Lee",
            "id": "graph-user-id",
            "mail": "ann@corp.example",
        })))
        .expect("a Graph profile parses");

        assert_eq!(profile.mail, "ann@corp.example");
        assert_eq!(profile.given_name.as_deref(), Some("Ann"));
        assert_eq!(profile.surname.as_deref(), Some("Lee"));

        // snake_case keys are not recognized: `given_name`/`surname` are
        // silently absent and the profile loses the names
        let profile: MicrosoftProfile = MicrosoftProfile::try_from(JsonValue::new(json!({
            "display_name": "Ann Lee",
            "given_name": "Ann",
            "surname": "Lee",
            "id": "graph-user-id",
            "mail": "ann@corp.example",
        })))
        .expect("required fields still present");
        assert!(
            profile.given_name.is_none(),
            "pinned: only camelCase givenName is deserialized"
        );
    }

    #[test]
    fn profile_requires_mail_as_a_string() {
        // BUG(pinned): MS Graph returns `"mail": null` for mailbox-less
        // guests, and `mail: String` makes the whole profile fail to parse —
        // such logins die at deserialization instead of falling back to
        // another identifier.
        assert!(
            MicrosoftProfile::try_from(JsonValue::new(json!({
                "displayName": "Guest User",
                "id": "graph-user-id",
                "mail": null,
            })))
            .is_err()
        );
        assert!(
            MicrosoftProfile::try_from(JsonValue::new(json!({
                "displayName": "Guest User",
                "id": "graph-user-id",
            })))
            .is_err()
        );
    }
}
