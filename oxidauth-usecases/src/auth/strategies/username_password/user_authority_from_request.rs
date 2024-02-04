use async_trait::async_trait;
use oxidauth_kernel::{
    auth::UserAuthorityFromRequest, error::BoxedError,
    user_authorities::create_user_authority::CreateUserAuthority, JsonValue,
};


use super::{
    authenticator::AuthenticateParams,
    helpers::{hash_password, raw_password_hash},
    UserAuthorityParams, UsernamePassword,
};

#[async_trait]
impl UserAuthorityFromRequest for UsernamePassword {
    #[tracing::instrument(
        name = "user_authority from username_password",
        skip(self)
    )]
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

        let password_hash =
            hash_password(password).map_err(|err| err.to_string())?;
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
