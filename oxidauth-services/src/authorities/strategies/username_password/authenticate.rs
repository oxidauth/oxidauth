use std::env::var as get_var;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use serde::{Deserialize, Serialize};

use super::*;

impl AuthenticateStrategy<UsernamePasswordAuthenticateInputs> for UsernamePasswordStrategy {
    type AuthorityParams = UsernamePasswordAuthorityParams;
    type UserAuthorityParams = UsernamePasswordUserAuthorityParams;

    fn authenticate(
        &self,
        authority_params: Self::AuthorityParams,
        user_authority_params: Self::UserAuthorityParams,
        params: UsernamePasswordAuthenticateInputs,
    ) -> Result<(), AuthenticateStrategyError> {
        let addtl_pepper = get_var(authority_params.pepper_env_var_key)
            .map_err(|_| AuthenticateStrategyError {})?;

        let password = format!(
            "{}:{}:{}:{}",
            &params.username, &params.password, authority_params.pepper, addtl_pepper
        );

        let password_hash = PasswordHash::new(&user_authority_params.password_hash)
            .map_err(|_| AuthenticateStrategyError {})?;

        Argon2::default()
            .verify_password(&password.into_bytes(), &password_hash)
            .map_err(|_| AuthenticateStrategyError {})?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsernamePasswordAuthenticateInputs {
    pub username: String,
    pub password: String,
}
