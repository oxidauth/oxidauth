use async_trait::async_trait;
use oxidauth_kernel::{
    JsonValue,
    auth::Authenticator,
    authorities::{Authority, UserAuthority},
    error::BoxedError,
};
use serde::Deserialize;

use super::{AuthorityParams, OAuth2};

#[derive(Clone, Deserialize)]
pub struct AuthenticateParams {
    pub email: String,
}

impl TryFrom<JsonValue> for AuthenticateParams {
    type Error = BoxedError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        let params = serde_json::from_value(value.inner_value())?;

        Ok(params)
    }
}

#[async_trait]
impl Authenticator for OAuth2 {
    #[tracing::instrument(name = "oauth2 authenticate", skip(self))]
    async fn authenticate(
        &self,
        _authenticate_params: JsonValue,
        authority: &Authority,
        user_authority: &UserAuthority,
    ) -> Result<(), BoxedError> {
        Ok(())
    }
}

#[tracing::instrument(name = "new oauth2 authenticator")]
pub async fn new(authority: &Authority) -> Result<Box<dyn Authenticator>, BoxedError> {
    let params: AuthorityParams = authority
        .params
        .clone()
        .try_into()?;

    Ok(Box::new(OAuth2 {
        authority_id: authority.id,
        params,
    }))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chrono::Utc;
    use oxidauth_kernel::{
        authorities::{
            Authority,
            AuthoritySettings,
            AuthorityStatus,
            AuthorityStrategy,
            TotpSettings,
        },
        jwt::EntitlementsEncoding,
        user_authorities::UserAuthority,
    };
    use serde_json::json;
    use uuid::Uuid;

    use super::{
        super::{
            OAuthFlavors,
            fixtures::{AUTHORITY_ID, CLIENT_KEY, oauth2},
        },
        *,
    };

    fn fixture_authority() -> Authority {
        Authority {
            id: AUTHORITY_ID,
            name: "oauth2-authority".to_owned(),
            client_key: CLIENT_KEY,
            status: AuthorityStatus::Enabled,
            strategy: AuthorityStrategy::Oauth2,
            settings: AuthoritySettings {
                jwt_ttl: Duration::from_secs(900),
                jwt_nbf_offset: Default::default(),
                refresh_token_ttl: Duration::from_secs(86_400),
                totp: TotpSettings::Disabled,
                entitlements_encoding: EntitlementsEncoding::Txt,
            },
            params: JsonValue::empty(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn fixture_user_authority() -> UserAuthority {
        UserAuthority {
            user_id: Uuid::new_v4(),
            authority_id: AUTHORITY_ID,
            user_identifier: "oauth@example.com".to_owned(),
            params: JsonValue::empty(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn authenticate_accepts_anything_after_the_idp_exchange_succeeded() {
        // BUG(pinned): audit B2.2 expected rejection cases, but the actual
        // OAuth2 authenticator verifies nothing — it ignores the params
        // entirely and trusts that the caller already completed the IdP
        // code exchange. Pinned as-is.
        let strategy = oauth2(OAuthFlavors::Google);
        let authority = fixture_authority();
        let user_authority = fixture_user_authority();

        for params in [
            JsonValue::new(json!({ "email": "someone@example.com" })),
            JsonValue::new(json!({})),
            JsonValue::new(json!("not even an object")),
            JsonValue::empty(),
        ] {
            let res = strategy
                .authenticate(params, &authority, &user_authority)
                .await;
            assert!(res.is_ok(), "actual behavior: no params are checked");
        }
    }

    #[tokio::test]
    async fn authenticate_params_extract_the_email_or_error() {
        let params: AuthenticateParams = JsonValue::new(json!({
            "email": "profile@example.com",
            "extra": "ignored",
        }))
        .try_into()
        .expect("email param parses");
        assert_eq!(params.email, "profile@example.com");

        let res = AuthenticateParams::try_from(JsonValue::new(json!({ "name": "x" })));
        let err = match res {
            Ok(_) => panic!("missing email must error"),
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains("missing field `email`"),
            "expected serde missing-field error, got: {err}"
        );
    }
}
