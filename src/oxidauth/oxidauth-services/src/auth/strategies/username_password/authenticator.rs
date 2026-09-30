use async_trait::async_trait;
use oxidauth_kernel::{
    JsonValue,
    Password,
    auth::Authenticator,
    authorities::Authority,
    error::BoxedError,
    user_authorities::UserAuthority,
};
use serde::Deserialize;

use super::{
    AuthorityParams,
    UserAuthorityParams,
    UsernamePassword,
    helpers::{raw_password_hash, verify_password},
};

#[derive(Clone, Deserialize)]
pub struct AuthenticateParams {
    pub username: String,
    pub password: Password,
}

impl TryFrom<JsonValue> for AuthenticateParams {
    type Error = BoxedError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        let s: Self = serde_json::from_value(value.inner_value())?;

        Ok(s)
    }
}

#[async_trait]
impl Authenticator for UsernamePassword {
    #[tracing::instrument(name = "username_password authenticate", skip(self))]
    async fn authenticate(
        &self,
        authenticate_params: JsonValue,
        authority: &Authority,
        user_authority: &UserAuthority,
    ) -> Result<(), BoxedError> {
        let authenticate_params: AuthenticateParams = authenticate_params.try_into()?;

        let password = raw_password_hash(
            &authenticate_params
                .password
                .inner_value(),
            &self.params.password_salt,
            &self.password_pepper,
        );

        let user_authority_params: UserAuthorityParams = user_authority
            .params
            .clone()
            .try_into()?;

        verify_password(password, user_authority_params.password_hash)
            .map_err(|err| err.to_string())?;

        Ok(())
    }
}

#[tracing::instrument(name = "new username_password authenticator")]
pub async fn new(authority: &Authority) -> Result<Box<dyn Authenticator>, BoxedError> {
    let params: AuthorityParams = authority
        .params
        .clone()
        .try_into()?;

    let password_pepper = std::env::var("OXIDAUTH_USERNAME_PASSWORD_PEPPER")?;

    let authority_id = authority.id;

    Ok(Box::new(UsernamePassword {
        authority_id,
        params,
        password_pepper,
    }))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        super::{
            fixtures::{
                PASSWORD_PEPPER,
                PASSWORD_SALT,
                authority,
                authority_params_json,
                stored_password_hash,
                user_authority,
                username_password,
                username_password_with,
            },
            helpers::hash_password,
        },
        *,
    };
    use crate::EnvGuard;

    const PASSWORD: &str = "s3cret-correct-password";

    fn authenticate_params(password: &str) -> JsonValue {
        JsonValue::new(json!({
            "username": "test-user",
            "password": password,
        }))
    }

    #[tokio::test]
    async fn authenticates_the_correct_password() {
        let strategy = username_password();
        let stored_hash = stored_password_hash(PASSWORD, PASSWORD_SALT, PASSWORD_PEPPER);
        let user_authority =
            user_authority(JsonValue::new(json!({ "password_hash": stored_hash })));

        let res = strategy
            .authenticate(
                authenticate_params(PASSWORD),
                &authority(authority_params_json()),
                &user_authority,
            )
            .await;

        assert!(
            res.is_ok(),
            "correct password must authenticate, got {res:?}"
        );
    }

    #[tokio::test]
    async fn rejects_the_wrong_password() {
        let strategy = username_password();
        let stored_hash = stored_password_hash(PASSWORD, PASSWORD_SALT, PASSWORD_PEPPER);
        let user_authority =
            user_authority(JsonValue::new(json!({ "password_hash": stored_hash })));

        let err = strategy
            .authenticate(
                authenticate_params("definitely-wrong"),
                &authority(authority_params_json()),
                &user_authority,
            )
            .await
            .expect_err("wrong password must not authenticate");

        // argon2 `Error::Password` surfaces through the `err.to_string()` map
        assert!(
            err.to_string()
                .contains("invalid password"),
            "expected an argon2 password-mismatch error, got: {err}"
        );
    }

    #[tokio::test]
    async fn rejects_a_hash_made_with_a_different_salt_or_pepper() {
        // the stored hash was produced with other (salt, pepper) material:
        // the authenticator must combine its own configured salt+pepper, not
        // trust the plaintext alone
        let strategy = username_password();
        let foreign_hash = stored_password_hash(PASSWORD, "not-my-salt", "not-my-pepper");
        let user_authority =
            user_authority(JsonValue::new(json!({ "password_hash": foreign_hash })));

        let res = strategy
            .authenticate(
                authenticate_params(PASSWORD),
                &authority(authority_params_json()),
                &user_authority,
            )
            .await;

        assert!(
            res.is_err(),
            "a foreign salt/pepper hash must not authenticate"
        );
    }

    #[tokio::test]
    async fn rejects_malformed_authenticate_params() {
        let strategy = username_password();
        let user_authority =
            user_authority(JsonValue::new(json!({ "password_hash": "irrelevant" })));
        let authority = authority(authority_params_json());

        // missing `password`
        let err = strategy
            .authenticate(
                JsonValue::new(json!({ "username": "u" })),
                &authority,
                &user_authority,
            )
            .await
            .expect_err("missing password field must error");
        assert!(
            err.to_string()
                .contains("missing field `password`"),
            "expected serde missing-field error, got: {err}"
        );

        // not even an object
        assert!(
            strategy
                .authenticate(JsonValue::new(json!("nope")), &authority, &user_authority)
                .await
                .is_err(),
            "non-object params must error"
        );

        // `password` present but not a string (Password deserializes from String)
        assert!(
            strategy
                .authenticate(
                    JsonValue::new(json!({ "username": "u", "password": 42 })),
                    &authority,
                    &user_authority
                )
                .await
                .is_err(),
            "non-string password must error"
        );
    }

    #[tokio::test]
    async fn rejects_malformed_stored_user_authority_params() {
        let strategy = username_password();

        // missing `password_hash` key entirely
        let missing_hash_ua = user_authority(JsonValue::new(json!({ "not_password_hash": 1 })));
        assert!(
            strategy
                .authenticate(
                    authenticate_params(PASSWORD),
                    &authority(authority_params_json()),
                    &missing_hash_ua,
                )
                .await
                .is_err(),
            "user authority params without password_hash must error"
        );

        // `password_hash` present but not a PHC argon2 string
        let bad_hash_ua = user_authority(JsonValue::new(json!({ "password_hash": "!!!" })));
        let err = strategy
            .authenticate(
                authenticate_params(PASSWORD),
                &authority(authority_params_json()),
                &bad_hash_ua,
            )
            .await
            .expect_err("malformed stored hash must error");
        assert!(
            err.to_string()
                .contains("password hash string"),
            "expected a PHC parse error, got: {err}"
        );
    }

    #[tokio::test]
    async fn new_reads_the_pepper_env_var_and_produces_a_working_authenticator() {
        let _pepper = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PASSWORD_PEPPER);

        let authority = authority(authority_params_json());
        let authenticator = new(&authority)
            .await
            .expect("authenticator builds");

        let stored_hash =
            hash_password(raw_password_hash(PASSWORD, PASSWORD_SALT, PASSWORD_PEPPER))
                .expect("argon2 hashing should succeed");
        let user_authority =
            user_authority(JsonValue::new(json!({ "password_hash": stored_hash })));

        assert!(
            authenticator
                .authenticate(authenticate_params(PASSWORD), &authority, &user_authority)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn new_errors_on_malformed_authority_params() {
        let _pepper = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PASSWORD_PEPPER);

        let authority = authority(JsonValue::new(json!({ "not_password_salt": true })));

        let res = new(&authority).await;
        let err = match res {
            Ok(_) => panic!("authority params without password_salt must error"),
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains("missing field `password_salt`"),
            "expected serde missing-field error, got: {err}"
        );
    }

    #[tokio::test]
    async fn authenticate_params_round_trip_username_and_password() {
        let params: AuthenticateParams = JsonValue::new(json!({
            "username": "round-trip-user",
            "password": "hunter2",
        }))
        .try_into()
        .expect("valid params deserialize");

        assert_eq!(params.username, "round-trip-user");
        assert_eq!(params.password.inner_value(), "hunter2");
    }

    #[tokio::test]
    async fn a_strategy_built_with_a_different_salt_rejects_the_same_password() {
        // guards against the strategy ignoring its configured salt at runtime
        let honest = username_password();
        let stored_hash = honest
            .authenticate(
                authenticate_params(PASSWORD),
                &authority(authority_params_json()),
                &user_authority(JsonValue::new(json!({
                    "password_hash": stored_password_hash(PASSWORD, PASSWORD_SALT, PASSWORD_PEPPER),
                }))),
            )
            .await;
        assert!(
            stored_hash.is_ok(),
            "baseline sanity check, got {stored_hash:?}"
        );

        let other = username_password_with("rotated-salt", PASSWORD_PEPPER);
        let res = other
            .authenticate(
                authenticate_params(PASSWORD),
                &authority(authority_params_json()),
                &user_authority(JsonValue::new(json!({
                    "password_hash": stored_password_hash(PASSWORD, PASSWORD_SALT, PASSWORD_PEPPER),
                }))),
            )
            .await;

        assert!(res.is_err(), "salt rotation must reject existing hashes");
    }
}
