use async_trait::async_trait;
use oxidauth_kernel::{JsonValue, auth::UserIdentifierFromRequest, error::BoxedError};

use super::{UsernamePassword, authenticator::AuthenticateParams};

#[async_trait]
impl UserIdentifierFromRequest for UsernamePassword {
    #[tracing::instrument(name = "user_identifier from username_password", skip(self))]
    async fn user_identifier_from_request(&self, params: &JsonValue) -> Result<String, BoxedError> {
        let AuthenticateParams { username, .. } = params.clone().try_into()?;

        Ok(username)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{super::fixtures::username_password, *};

    #[tokio::test]
    async fn extracts_the_username_ignoring_extra_keys() {
        let strategy = username_password();

        let identifier = strategy
            .user_identifier_from_request(&JsonValue::new(json!({
                "username": "extracted-user",
                "password": "anything",
                "unrelated": true,
            })))
            .await
            .expect("username must be extractable");

        assert_eq!(identifier, "extracted-user");
    }

    #[tokio::test]
    async fn errors_when_the_username_is_missing_or_wrong_type() {
        let strategy = username_password();

        let err = strategy
            .user_identifier_from_request(&JsonValue::new(json!({ "password": "pw" })))
            .await
            .expect_err("missing username must error");
        assert!(
            err.to_string()
                .contains("missing field `username`"),
            "expected serde missing-field error, got: {err}"
        );

        assert!(
            strategy
                .user_identifier_from_request(&JsonValue::new(json!({
                    "username": 7,
                    "password": "pw",
                })))
                .await
                .is_err(),
            "non-string username must error"
        );
    }
}
