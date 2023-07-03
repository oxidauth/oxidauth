use std::env::var as get_var;

use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

use crate::dev_prelude::*;

use super::{
    RegisterStrategy, RegisterStrategyError, UsernamePasswordAuthorityParams,
    UsernamePasswordStrategy, UsernamePasswordUserAuthorityParams,
};

impl RegisterStrategy<UsernamePasswordRegisterInputs> for UsernamePasswordStrategy {
    type UserAuthorityParams = UsernamePasswordUserAuthorityParams;

    fn user_authority_params(
        &self,
        authority_params: Value,
        params: UsernamePasswordRegisterInputs,
    ) -> Result<Self::UserAuthorityParams, RegisterStrategyError> {
        let authority_params: UsernamePasswordAuthorityParams =
            serde_json::from_value(authority_params).map_err(|_| RegisterStrategyError {})?;

        let addtl_pepper =
            get_var(authority_params.pepper_env_var_key).map_err(|_| RegisterStrategyError {})?;

        let password = format!(
            "{}:{}:{}:{}",
            &params.username, &params.password, authority_params.pepper, addtl_pepper
        );

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(&password.into_bytes(), &salt)
            .map_err(|_| RegisterStrategyError {})?
            .to_string();

        let user_authority_params = UsernamePasswordUserAuthorityParams { password_hash };

        Ok(user_authority_params)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsernamePasswordRegisterInputs {
    pub username: String,
    pub password: String,
}
