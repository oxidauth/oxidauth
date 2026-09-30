use async_trait::async_trait;
use oxidauth_kernel::{
    JsonValue,
    auth::UserAuthorityFromRequest,
    error::BoxedError,
    user_authorities::create_user_authority::CreateUserAuthority,
};

use super::{
    UserAuthorityParams,
    UsernamePassword,
    authenticator::AuthenticateParams,
    helpers::{hash_password, raw_password_hash},
};

#[async_trait]
impl UserAuthorityFromRequest for UsernamePassword {
    #[tracing::instrument(name = "user_authority from username_password", skip(self))]
    async fn user_authority_from_request(
        &self,
        params: JsonValue,
    ) -> Result<CreateUserAuthority, BoxedError> {
        let authority_params: AuthenticateParams = params.clone().try_into()?;

        let password = raw_password_hash(
            &authority_params
                .password
                .inner_value(),
            &self.params.password_salt,
            &self.password_pepper,
        );

        let password_hash = hash_password(password).map_err(|err| err.to_string())?;
        let params = UserAuthorityParams { password_hash };

        let params = serde_json::to_value(params)?;

        let user_identifier = authority_params
            .username
            .clone();

        let user_authority = CreateUserAuthority {
            authority_id: self.authority_id,
            user_identifier,
            params: JsonValue::new(params),
        };

        Ok(user_authority)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        super::fixtures::{AUTHORITY_ID, PASSWORD_PEPPER, PASSWORD_SALT, username_password},
        *,
    };
    use crate::auth::strategies::username_password::helpers::verify_password;

    #[tokio::test]
    async fn builds_a_peppered_hash_user_authority_keyed_by_username() {
        let strategy = username_password();

        let created = strategy
            .user_authority_from_request(JsonValue::new(json!({
                "username": "identifier-user",
                "password": "pass-word-2",
            })))
            .await
            .expect("valid credentials must build a user authority");

        assert_eq!(created.authority_id, AUTHORITY_ID);
        assert_eq!(created.user_identifier, "identifier-user");

        let params: UserAuthorityParams = created
            .params
            .try_into()
            .expect("params carry password_hash");
        assert_eq!(
            verify_password(
                raw_password_hash("pass-word-2", PASSWORD_SALT, PASSWORD_PEPPER),
                params.password_hash,
            ),
            Ok(true),
            "stored hash must verify the salt+pepper raw material"
        );
    }

    #[tokio::test]
    async fn errors_on_missing_or_malformed_request_fields() {
        let strategy = username_password();

        let err = strategy
            .user_authority_from_request(JsonValue::new(json!({ "username": "no-password" })))
            .await
            .expect_err("missing password must error");
        assert!(
            err.to_string()
                .contains("missing field `password`"),
            "expected serde missing-field error, got: {err}"
        );

        assert!(
            strategy
                .user_authority_from_request(JsonValue::new(json!([])))
                .await
                .is_err(),
            "non-object params must error"
        );
    }
}
