use std::env::var as get_var;

use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use oxidauth_kernel::Password;
use rand_core::OsRng;
use serde::Deserialize;

use super::{
    RegisterStrategy,
    RegisterStrategyError,
    UsernamePasswordAuthorityParams,
    UsernamePasswordStrategy,
    UsernamePasswordUserAuthorityParams,
};
use crate::dev_prelude::*;

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
            params.username,
            params.password.inner_value(),
            authority_params.pepper,
            addtl_pepper
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

#[derive(Debug, Deserialize)]
pub struct UsernamePasswordRegisterInputs {
    pub username: String,
    pub password: Password,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testing_manual_password_debug_impl() {
        let username_password_inputs = UsernamePasswordRegisterInputs {
            username: "username".to_owned(),
            password: Password::new("super_secret_password".to_owned()),
        };

        assert_eq!(
            format!("{username_password_inputs:?}"),
            "UsernamePasswordRegisterInputs { username: \"username\", password: ****** }"
        );
    }

    use serde_json::json;

    use super::super::{AuthenticateStrategy, authenticate::UsernamePasswordAuthenticateInputs};
    use crate::EnvGuard;

    const PEPPER_ENV_KEY: &str = "OXIDAUTH_UNIT_TEST_B16R_PEPPER";
    const RUNTIME_PEPPER: &str = "runtime-pepper-value";
    const AUTHORITY_PEPPER: &str = "authority-pepper-value";
    const USERNAME: &str = "shared-vector-user";
    const PASSWORD: &str = "shared-vector-password";

    fn authority_params_json() -> serde_json::Value {
        json!({
            "pepper": AUTHORITY_PEPPER,
            "pepper_env_var_key": PEPPER_ENV_KEY,
        })
    }

    #[test]
    fn register_side_hash_is_accepted_by_the_authenticate_side() {
        // the register strategy (username:password:pepper:addtl_pepper from
        // the env key) and the authenticate strategy must agree on the raw
        // material — drift would silently lock out every newly registered
        // user
        let _env = EnvGuard::set(PEPPER_ENV_KEY, RUNTIME_PEPPER);

        let stored = UsernamePasswordStrategy {}
            .user_authority_params(
                authority_params_json(),
                UsernamePasswordRegisterInputs {
                    username: USERNAME.to_owned(),
                    password: Password::new(PASSWORD.to_owned()),
                },
            )
            .expect("registration must hash the shared raw material");
        assert!(
            stored
                .password_hash
                .starts_with("$argon2")
        );
        let password_hash = stored.password_hash;

        let strategy = UsernamePasswordStrategy {};
        let authenticate_res = strategy.authenticate(
            serde_json::from_value(authority_params_json())
                .expect("authority params fixture parses"),
            UsernamePasswordUserAuthorityParams {
                password_hash: password_hash.clone(),
            },
            UsernamePasswordAuthenticateInputs {
                username: USERNAME.to_owned(),
                password: Password::new(PASSWORD.to_owned()),
            },
        );

        assert!(
            authenticate_res.is_ok(),
            "register-side hash must authenticate: {authenticate_res:?}"
        );

        let wrong_username_res = strategy.authenticate(
            serde_json::from_value(authority_params_json())
                .expect("authority params fixture parses"),
            UsernamePasswordUserAuthorityParams { password_hash },
            UsernamePasswordAuthenticateInputs {
                username: "somebody-else".to_owned(),
                password: Password::new(PASSWORD.to_owned()),
            },
        );

        assert!(
            wrong_username_res.is_err(),
            "the register-side hash must be bound to the registering username"
        );
    }
}
