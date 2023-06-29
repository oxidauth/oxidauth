pub mod authenticate;
pub mod register;

use serde::{Deserialize, Serialize};

use oxidauth_repository::authorities::query_authority_by_client_id::QueryAuthorityByClientId;

const USERNAME_PASSWORD_ADDTL_PEPPER: &str = "USERNAME_PASSWORD_ADDTL_PEPPER";

pub struct RegisterUseCase<R>
where
    R: QueryAuthorityByClientId,
{
    repo: R,
}

mod helpers {
    use argon2::{
        password_hash::{Error as PasswordHashError, SaltString},
        Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    };
    use rand_core::OsRng;

    pub fn hash_password(password: String) -> Result<String, PasswordHashError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(&password.into_bytes(), &salt)?
            .to_string();

        Ok(password_hash)
    }

    pub fn verify_password(
        password: String,
        password_hash: String,
    ) -> Result<bool, PasswordHashError> {
        let password_hash = PasswordHash::new(&password_hash)?;

        Argon2::default().verify_password(&password.into_bytes(), &password_hash)?;

        Ok(true)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsernamePasswordUserAuthorityParams {
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsernamePasswordAuthorityParams {
    pub password_pepper: String,
}
