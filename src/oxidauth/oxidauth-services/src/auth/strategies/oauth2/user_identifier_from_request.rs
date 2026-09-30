use async_trait::async_trait;
use oxidauth_kernel::{JsonValue, auth::UserIdentifierFromRequest, error::BoxedError};

use super::{OAuth2, authenticator::AuthenticateParams};

#[async_trait]
impl UserIdentifierFromRequest for OAuth2 {
    #[tracing::instrument(name = "user_identifier from oauth2", skip(self))]
    async fn user_identifier_from_request(&self, params: &JsonValue) -> Result<String, BoxedError> {
        let AuthenticateParams { email, .. } = params.clone().try_into()?;

        Ok(email)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        super::{OAuthFlavors, fixtures::oauth2},
        *,
    };

    #[tokio::test]
    async fn extracts_the_email_from_profile_params() {
        let strategy = oauth2(OAuthFlavors::Google);

        let identifier = strategy
            .user_identifier_from_request(&JsonValue::new(json!({
                "email": "profile@example.com",
                "given_name": "Ann",
            })))
            .await
            .expect("email must be extractable");

        assert_eq!(identifier, "profile@example.com");
    }

    #[tokio::test]
    async fn errors_when_the_email_is_missing_or_wrong_type() {
        let strategy = oauth2(OAuthFlavors::Microsoft);

        let err = strategy
            .user_identifier_from_request(&JsonValue::new(json!({ "name": "no email" })))
            .await
            .expect_err("missing email must error");
        assert!(
            err.to_string()
                .contains("missing field `email`"),
            "expected serde missing-field error, got: {err}"
        );

        assert!(
            strategy
                .user_identifier_from_request(&JsonValue::new(json!({ "email": 42 })))
                .await
                .is_err(),
            "non-string email must error"
        );
    }
}
