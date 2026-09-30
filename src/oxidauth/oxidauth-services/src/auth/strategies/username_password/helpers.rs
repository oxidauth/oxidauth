use argon2::{
    Argon2,
    PasswordHash,
    PasswordVerifier,
    password_hash::{Error as PasswordHashError, PasswordHasher, SaltString},
};
use rand_core::OsRng;

pub fn verify_password(password: String, password_hash: String) -> Result<bool, PasswordHashError> {
    let password_hash = PasswordHash::new(&password_hash)?;

    Argon2::default().verify_password(&password.into_bytes(), &password_hash)?;

    Ok(true)
}

pub fn raw_password_hash(password: &str, password_salt: &str, password_pepper: &str) -> String {
    format!("{}:{}:{}", password, password_salt, password_pepper)
}

pub fn hash_password(password: String) -> Result<String, PasswordHashError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(&password.into_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_password_hash_joins_password_salt_pepper_with_colons() {
        let raw = raw_password_hash("hunter2", "s-a-l-t", "p3pp3r");

        assert_eq!(raw, "hunter2:s-a-l-t:p3pp3r");
    }

    #[test]
    fn raw_password_hash_does_not_escape_colons_in_inputs() {
        // BUG(pinned): the `:` joiner is ambiguous — inputs containing `:`
        // can collide across field boundaries, e.g. ("a:b", "c", "d") and
        // ("a", "b:c", "d") produce the same raw material. Pinned as-is;
        // changing the format would invalidate every stored password hash.
        assert_eq!(raw_password_hash("a:b", "c", "d"), "a:b:c:d");
        assert_eq!(
            raw_password_hash("a:b", "c", "d"),
            raw_password_hash("a", "b:c", "d")
        );
    }

    #[test]
    fn hash_password_then_verify_password_accepts_the_same_material() {
        let raw = raw_password_hash("correct horse", "unit-test-salt", "unit-test-pepper");

        let hash = hash_password(raw.clone()).expect("argon2 hashing should succeed");

        assert!(
            hash.starts_with("$argon2"),
            "hash should be a PHC argon2 string"
        );
        assert!(
            !hash.contains("correct horse"),
            "raw material must not leak into hash"
        );
        assert_eq!(verify_password(raw, hash), Ok(true));
    }

    #[test]
    fn verify_password_rejects_the_wrong_password() {
        let hash = hash_password(raw_password_hash("right-material", "salt", "pepper"))
            .expect("argon2 hashing should succeed");
        let wrong = raw_password_hash("wrong-material", "salt", "pepper");

        assert!(matches!(
            verify_password(wrong, hash),
            Err(PasswordHashError::Password)
        ));
    }

    #[test]
    fn verify_password_rejects_a_malformed_hash() {
        assert!(matches!(
            verify_password("pw".to_owned(), "not-a-phc-hash".to_owned()),
            Err(PasswordHashError::PhcStringField)
        ));
        assert!(matches!(
            verify_password("pw".to_owned(), String::new()),
            Err(PasswordHashError::PhcStringField)
        ));
    }

    #[test]
    fn salt_and_pepper_are_part_of_the_verified_material() {
        let hash = hash_password(raw_password_hash("pw", "salt", "pepper"))
            .expect("argon2 hashing should succeed");

        // the bare password must not verify against the peppered hash
        assert!(matches!(
            verify_password("pw".to_owned(), hash.clone()),
            Err(PasswordHashError::Password)
        ));
        // a different pepper is a different password
        assert!(matches!(
            verify_password(
                raw_password_hash("pw", "salt", "other-pepper"),
                hash.clone()
            ),
            Err(PasswordHashError::Password)
        ));
        // a different salt is a different password
        assert!(matches!(
            verify_password(raw_password_hash("pw", "other-salt", "pepper"), hash),
            Err(PasswordHashError::Password)
        ));
    }
}
