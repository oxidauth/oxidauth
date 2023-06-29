use std::env;

use oxidauth_kernel::authorities::{
    AuthenticateError, AuthenticateParamsExtractor, AuthenticateParamsExtractorError,
    AuthenticateService, Authority, UserAuthority,
};

use crate::dev_prelude::*;

use super::*;

#[async_trait]
impl<R> AuthenticateService for RegisterUseCase<R>
where
    R: QueryAuthorityByClientId,
{
    async fn authenticate(&self) -> Result<(), AuthenticateError> {
        todo!()
    }
}

#[derive(Debug)]
pub struct UsernamePasswordAuthenticateParams {
    pub client_id: Uuid,
    pub username: String,
    pub password: String,
}

#[async_trait]
impl AuthenticateParamsExtractor for UsernamePasswordAuthenticateParams {
    async fn client_id(&self) -> Result<Uuid, AuthenticateParamsExtractorError> {
        Ok(self.client_id)
    }

    async fn user_identifier(&self) -> Result<String, AuthenticateParamsExtractorError> {
        Ok(self.username.clone())
    }

    async fn params(
        &self,
        authority: &Authority,
        user_authority: &UserAuthority,
    ) -> Result<Value, AuthenticateParamsExtractorError> {
        let authority_params: UsernamePasswordAuthorityParams =
            serde_json::from_value(authority.params.clone())
                .map_err(|_| AuthenticateParamsExtractorError {})?;

        let username_password_addtl_pepper = env::var(USERNAME_PASSWORD_ADDTL_PEPPER)
            .map_err(|_| AuthenticateParamsExtractorError {})?;

        let password = format!(
            "{}:{}:{}:{}",
            &self.username,
            &self.password,
            &authority_params.password_pepper,
            &username_password_addtl_pepper,
        );

        let user_authority_params: UsernamePasswordUserAuthorityParams =
            serde_json::from_value(user_authority.params.clone())
                .map_err(|_| AuthenticateParamsExtractorError {})?;

        helpers::verify_password(password, user_authority_params.password_hash)
            .map_err(|_| AuthenticateParamsExtractorError {})?;

        Ok(Value::Bool(true))
    }
}
