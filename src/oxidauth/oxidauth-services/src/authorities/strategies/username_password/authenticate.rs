use std::env::var as get_var;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use oxidauth_kernel::Password;
use serde::Deserialize;

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
            params.username,
            params.password.inner_value(),
            authority_params.pepper,
            addtl_pepper
        );

        let password_hash = PasswordHash::new(&user_authority_params.password_hash)
            .map_err(|_| AuthenticateStrategyError {})?;

        Argon2::default()
            .verify_password(&password.into_bytes(), &password_hash)
            .map_err(|_| AuthenticateStrategyError {})?;

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UsernamePasswordAuthenticateInputs {
    pub username: String,
    pub password: Password,
}

#[cfg(test)]
mod tests {
    use argon2::password_hash::{PasswordHasher, SaltString};
    use rand_core::OsRng;

    use super::*;
    use crate::EnvGuard;

    const PEPPER_ENV_KEY: &str = "OXIDAUTH_UNIT_TEST_B16_PEPPER";
    const RUNTIME_PEPPER: &str = "runtime-pepper-value";
    const AUTHORITY_PEPPER: &str = "authority-pepper-value";
    const USERNAME: &str = "authority-user";
    const PASSWORD: &str = "authority-password";

    fn strategy() -> UsernamePasswordStrategy {
        UsernamePasswordStrategy {}
    }

    fn authority_params(env_key: &str) -> UsernamePasswordAuthorityParams {
        UsernamePasswordAuthorityParams {
            pepper: AUTHORITY_PEPPER.to_owned(),
            pepper_env_var_key: env_key.to_owned(),
        }
    }

    fn inputs(password: &str) -> UsernamePasswordAuthenticateInputs {
        UsernamePasswordAuthenticateInputs {
            username: USERNAME.to_owned(),
            password: Password::new(password.to_owned()),
        }
    }

    /// The documented raw material: `username:password:pepper:addtl_pepper`.
    fn stored_hash(username: &str, password: &str, pepper: &str, addtl_pepper: &str) -> String {
        let raw = format!("{username}:{password}:{pepper}:{addtl_pepper}");
        let salt = SaltString::generate(&mut OsRng);

        Argon2::default()
            .hash_password(raw.as_bytes(), &salt)
            .expect("argon2 hashing should succeed")
            .to_string()
    }

    fn user_params(password_hash: String) -> UsernamePasswordUserAuthorityParams {
        UsernamePasswordUserAuthorityParams { password_hash }
    }

    #[test]
    fn authenticates_the_correct_password() {
        let _env = EnvGuard::set(PEPPER_ENV_KEY, RUNTIME_PEPPER);

        let res = strategy().authenticate(
            authority_params(PEPPER_ENV_KEY),
            user_params(stored_hash(
                USERNAME,
                PASSWORD,
                AUTHORITY_PEPPER,
                RUNTIME_PEPPER,
            )),
            inputs(PASSWORD),
        );

        assert!(res.is_ok(), "correct password must authenticate: {res:?}");
    }

    #[test]
    fn rejects_the_wrong_password() {
        let _env = EnvGuard::set(PEPPER_ENV_KEY, RUNTIME_PEPPER);

        let res = strategy().authenticate(
            authority_params(PEPPER_ENV_KEY),
            user_params(stored_hash(
                USERNAME,
                PASSWORD,
                AUTHORITY_PEPPER,
                RUNTIME_PEPPER,
            )),
            inputs("wrong-password"),
        );

        assert!(res.is_err(), "wrong password must not authenticate");
    }

    #[test]
    fn rejects_a_hash_made_without_the_username_in_the_raw_material() {
        // pins the `username:` prefix as part of the hashed material: two
        // users sharing one password must not authenticate for each other
        let _env = EnvGuard::set(PEPPER_ENV_KEY, RUNTIME_PEPPER);

        let res = strategy().authenticate(
            authority_params(PEPPER_ENV_KEY),
            user_params(stored_hash(
                "somebody-else",
                PASSWORD,
                AUTHORITY_PEPPER,
                RUNTIME_PEPPER,
            )),
            inputs(PASSWORD),
        );

        assert!(res.is_err(), "raw material must include the username");
    }

    #[test]
    fn rejects_a_hash_made_with_a_different_pepper() {
        let _env = EnvGuard::set(PEPPER_ENV_KEY, RUNTIME_PEPPER);

        let res = strategy().authenticate(
            authority_params(PEPPER_ENV_KEY),
            user_params(stored_hash(
                USERNAME,
                PASSWORD,
                "other-pepper",
                RUNTIME_PEPPER,
            )),
            inputs(PASSWORD),
        );

        assert!(
            res.is_err(),
            "authority pepper must be part of the material"
        );
    }

    #[test]
    fn rejects_a_hash_made_without_the_env_pepper() {
        let _env = EnvGuard::set(PEPPER_ENV_KEY, RUNTIME_PEPPER);

        let res = strategy().authenticate(
            authority_params(PEPPER_ENV_KEY),
            user_params(stored_hash(
                USERNAME,
                PASSWORD,
                AUTHORITY_PEPPER,
                "stale-pepper",
            )),
            inputs(PASSWORD),
        );

        assert!(
            res.is_err(),
            "the env-referenced additional pepper must be part of the material"
        );
    }

    #[test]
    fn rejects_a_malformed_stored_hash() {
        let _env = EnvGuard::set(PEPPER_ENV_KEY, RUNTIME_PEPPER);

        let res = strategy().authenticate(
            authority_params(PEPPER_ENV_KEY),
            user_params("not-a-phc-hash".to_owned()),
            inputs(PASSWORD),
        );

        assert!(res.is_err(), "malformed stored hash must error");
    }

    #[test]
    fn rejects_when_the_pepper_env_var_is_unset() {
        // name chosen so no environment defines it; ENV_LOCK is still held so
        // the strategy's `get_var` cannot race other tests' `set_var`
        let _outer = crate::ENV_LOCK
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let missing_key = "OXIDAUTH_UNIT_TEST_B16_PEPPER_DEFINITELY_UNSET";
        assert!(std::env::var(missing_key).is_err(), "test precondition");

        let res = strategy().authenticate(
            authority_params(missing_key),
            user_params(stored_hash(
                USERNAME,
                PASSWORD,
                AUTHORITY_PEPPER,
                RUNTIME_PEPPER,
            )),
            inputs(PASSWORD),
        );

        assert!(res.is_err(), "a missing pepper env var must fail the login");
    }
}
