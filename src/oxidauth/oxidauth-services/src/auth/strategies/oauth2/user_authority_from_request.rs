use async_trait::async_trait;
use oxidauth_kernel::{
    JsonValue,
    auth::UserAuthorityFromRequest,
    error::BoxedError,
    user_authorities::create_user_authority::CreateUserAuthority,
};

use super::{OAuth2, authenticator::AuthenticateParams};

#[async_trait]
impl UserAuthorityFromRequest for OAuth2 {
    #[tracing::instrument(name = "user_authority from oauth2 credentials", skip(self))]
    async fn user_authority_from_request(
        &self,
        params: JsonValue,
    ) -> Result<CreateUserAuthority, BoxedError> {
        let authority_params: AuthenticateParams = params.clone().try_into()?;

        let params = serde_json::to_value(params)?;

        let user_identifier = authority_params.email.clone();

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
        super::{
            OAuthFlavors,
            fixtures::{AUTHORITY_ID, oauth2},
        },
        *,
    };

    #[tokio::test]
    async fn keys_the_user_authority_by_email_with_input_params_echoed() {
        let strategy = oauth2(OAuthFlavors::Google);
        let input = JsonValue::new(json!({
            "email": "callback@example.com",
            "given_name": "Ann",
            "family_name": "Lee",
        }));

        let created = strategy
            .user_authority_from_request(input.clone())
            .await
            .expect("a profile with email builds a user authority");

        assert_eq!(created.authority_id, AUTHORITY_ID);
        // pinned: the callback path identifies by EMAIL (unlike
        // Registrar::register, which identifies by username)
        assert_eq!(created.user_identifier, "callback@example.com");
        assert_eq!(
            &*created.params, &*input,
            "pinned: params echo the full input profile, not a subset"
        );
    }

    #[tokio::test]
    async fn errors_when_the_email_is_missing_or_malformed() {
        let strategy = oauth2(OAuthFlavors::Microsoft);

        let err = strategy
            .user_authority_from_request(JsonValue::new(json!({ "name": "nope" })))
            .await
            .expect_err("missing email must error");
        assert!(
            err.to_string()
                .contains("missing field `email`"),
            "expected serde missing-field error, got: {err}"
        );

        assert!(
            strategy
                .user_authority_from_request(JsonValue::new(json!("")))
                .await
                .is_err(),
            "empty-string / non-object params must error"
        );
    }
}
