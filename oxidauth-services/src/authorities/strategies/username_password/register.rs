use std::env;

use oxidauth_kernel::authorities::{
    Authority, RegisterError, RegisterParamsExtractor, RegisterParamsExtractorError,
    RegisterService,
};
use oxidauth_repository::authorities::query_authority_by_client_id::QueryAuthorityByClientId;
use serde::{Deserialize, Serialize};

use crate::dev_prelude::*;

use super::*;

#[async_trait]
impl<R, P> RegisterService<P> for RegisterUseCase<R>
where
    R: QueryAuthorityByClientId,
    P: RegisterParamsExtractor,
{
    async fn register(&self, params: P) -> Result<(), RegisterError> {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsernamePasswordRegisterParams {
    pub client_id: Uuid,
    pub username: String,
    pub password: String,
}

#[async_trait]
impl RegisterParamsExtractor for UsernamePasswordRegisterParams {
    async fn client_id(&self) -> Result<Uuid, RegisterParamsExtractorError> {
        Ok(self.client_id)
    }

    async fn user_identifier(&self) -> Result<String, RegisterParamsExtractorError> {
        Ok(self.username.clone())
    }

    async fn params(&self, authority: &Authority) -> Result<Value, RegisterParamsExtractorError> {
        let authority_params: UsernamePasswordAuthorityParams =
            serde_json::from_value(authority.params.clone())
                .map_err(|_| RegisterParamsExtractorError {})?;

        let username_password_addtl_pepper = env::var(USERNAME_PASSWORD_ADDTL_PEPPER)
            .map_err(|_| RegisterParamsExtractorError {})?;

        let password = format!(
            "{}:{}:{}:{}",
            &self.username,
            &self.password,
            &authority_params.password_pepper,
            &username_password_addtl_pepper,
        );

        let password_hash =
            helpers::hash_password(password).map_err(|_| RegisterParamsExtractorError {})?;

        let user_authority_params = UsernamePasswordUserAuthorityParams { password_hash };

        let value = serde_json::to_value(user_authority_params)
            .map_err(|_| RegisterParamsExtractorError {})?;

        Ok(value)
    }
}
