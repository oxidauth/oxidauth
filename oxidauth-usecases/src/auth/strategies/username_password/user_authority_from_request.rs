use async_trait::async_trait;
use oxidauth_kernel::{
    auth::UserAuthorityFromRequest, error::BoxedError,
    user_authorities::create_user_authority::CreateUserAuthority,
};
use serde_json::Value;

use super::{
    authenticator::AuthenticateParams,
    helpers::{hash_password, raw_password_hash},
    UserAuthorityParams, UsernamePassword,
};

#[async_trait]
impl UserAuthorityFromRequest for UsernamePassword {
    async fn user_authority_from_request(
        &self,
        params: Value,
    ) -> Result<CreateUserAuthority, BoxedError> {
        let authority_params: AuthenticateParams = params.clone().try_into()?;

        let password = raw_password_hash(
            &authority_params.password,
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
            user_id: None,
            authority_id: self.authority_id,
            user_identifier,
            params,
        };

        Ok(user_authority)
    }
}
