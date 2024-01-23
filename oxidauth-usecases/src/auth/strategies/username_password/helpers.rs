use argon2::{
    password_hash::{Error as PasswordHashError, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use rand_core::OsRng;

pub fn verify_password(
    password: String,
    password_hash: String,
) -> Result<bool, PasswordHashError> {
    let password_hash = PasswordHash::new(&password_hash)?;

    Argon2::default().verify_password(
        &password.into_bytes(),
        &password_hash,
    )?;

    Ok(true)
}

pub fn raw_password_hash(
    password_salt: &str,
    password_pepper: &str,
    password: &str,
) -> String {
    format!(
        "{}:{}:{}",
        password, password_salt, password_pepper
    )
}

pub fn hash_password(password: String) -> Result<String, PasswordHashError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(&password.into_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}
