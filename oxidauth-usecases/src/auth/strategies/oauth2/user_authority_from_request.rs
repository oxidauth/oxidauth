use async_trait::async_trait;

use oxidauth_kernel::{
    JsonValue, auth::UserAuthorityFromRequest, error::BoxedError,
    user_authorities::create_user_authority::CreateUserAuthority,
};

use super::{OAuth2, OAuth2AuthorityParams, authenticator::AuthenticateParams};

#[async_trait]
impl UserAuthorityFromRequest for OAuth2 {
    #[tracing::instrument(name = "user_authority from oauth2 credentials", skip(self))]
    async fn user_authority_from_request(
        &self,
        params: JsonValue,
    ) -> Result<CreateUserAuthority, BoxedError> {
        let authority_params: AuthenticateParams = params.clone().try_into()?;

        let params = OAuth2AuthorityParams {
            access_token: authority_params.access_token,
        };

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
