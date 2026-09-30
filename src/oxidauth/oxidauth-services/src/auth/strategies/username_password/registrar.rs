use async_trait::async_trait;
use oxidauth_kernel::{
    JsonValue,
    Password,
    auth::Registrar,
    authorities::Authority,
    error::BoxedError,
    user_authorities::create_user_authority::CreateUserAuthority,
    users::{UserKind, UserStatus, create_user::CreateUser},
};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use super::{
    AuthorityParams,
    UserAuthorityParams,
    UsernamePassword,
    helpers::{hash_password, raw_password_hash},
};

#[async_trait]
impl Registrar for UsernamePassword {
    #[tracing::instrument(name = "username_password register", skip(self))]
    async fn register(
        &self,
        register_params: JsonValue,
    ) -> Result<(CreateUser, CreateUserAuthority), BoxedError> {
        let register_params: UsernamePasswordRegisterParams = register_params
            .clone()
            .try_into()?;

        let password = register_params
            .password
            .clone()
            .inner_value();
        let password_confirmation = register_params
            .password_confirmation
            .clone()
            .inner_value();

        if password != password_confirmation {
            return Err("password and password confirmation do not match".into());
        }

        let user: CreateUser = register_params.clone().into();

        let password =
            raw_password_hash(&password, &self.params.password_salt, &self.password_pepper);

        let password_hash = hash_password(password).map_err(|err| err.to_string())?;

        let params = UserAuthorityParams { password_hash };

        let params = serde_json::to_value(params)?;

        let user_authority = CreateUserAuthority {
            authority_id: self.authority_id,
            user_identifier: user.username.clone(),
            params: JsonValue::new(params),
        };

        Ok((user, user_authority))
    }
}

pub async fn new(authority: &Authority) -> Result<Box<dyn Registrar>, BoxedError> {
    let params: AuthorityParams = authority
        .params
        .clone()
        .try_into()?;
    let authority_id = authority.id;
    let password_pepper = std::env::var("OXIDAUTH_USERNAME_PASSWORD_PEPPER")?;

    Ok(Box::new(UsernamePassword {
        authority_id,
        params,
        password_pepper,
    }))
}

#[derive(Clone, Deserialize)]
pub struct UsernamePasswordRegisterParams {
    pub username: String,
    pub password: Password,
    pub password_confirmation: Password,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub kind: Option<UserKind>,
}

impl TryFrom<JsonValue> for UsernamePasswordRegisterParams {
    type Error = BoxedError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        let s: Self = serde_json::from_value(value.inner_value())?;

        Ok(s)
    }
}

impl From<UsernamePasswordRegisterParams> for CreateUser {
    fn from(params: UsernamePasswordRegisterParams) -> Self {
        let UsernamePasswordRegisterParams {
            username,
            email,
            first_name,
            last_name,
            kind,
            ..
        } = params.clone();
        let user_id = Uuid::new_v4();
        let kind = Some(kind.unwrap_or_default());

        Self {
            id: Some(user_id),
            username,
            email,
            first_name,
            last_name,
            status: Some(UserStatus::default()),
            kind,
            profile: Some(Value::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use oxidauth_kernel::users::UserKind;
    use serde_json::json;

    use super::{
        super::fixtures::{
            AUTHORITY_ID,
            PASSWORD_PEPPER,
            PASSWORD_SALT,
            authority,
            authority_params_json,
            username_password,
        },
        *,
    };
    use crate::{EnvGuard, auth::strategies::username_password::helpers::verify_password};

    fn register_params(extra: serde_json::Value) -> JsonValue {
        let mut base = json!({
            "username": "new-user",
            "password": "pass-word-1",
            "password_confirmation": "pass-word-1",
        });

        for (key, value) in extra
            .as_object()
            .expect("extra is an object")
        {
            base[key] = value.clone();
        }

        JsonValue::new(base)
    }

    #[tokio::test]
    async fn registers_with_a_peppered_argon2_password_hash() {
        let strategy = username_password();

        let (user, user_authority) = strategy
            .register(register_params(json!({
                "email": "new@example.com",
                "first_name": "New",
                "last_name": "User",
            })))
            .await
            .expect("valid register params must succeed");

        assert_eq!(user.username, "new-user");
        assert_eq!(user.email.as_deref(), Some("new@example.com"));
        assert_eq!(user.first_name.as_deref(), Some("New"));
        assert_eq!(user.last_name.as_deref(), Some("User"));
        assert!(user.id.is_some(), "registration pre-assigns a user id");
        assert!(
            matches!(user.kind, Some(UserKind::Human)),
            "kind defaults to human, got {:?}",
            user.kind
        );
        assert!(
            matches!(user.status, Some(UserStatus::Enabled)),
            "status defaults to enabled, got {:?}",
            user.status
        );

        assert_eq!(user_authority.authority_id, AUTHORITY_ID);
        assert_eq!(
            user_authority.user_identifier, "new-user",
            "user_identifier is the username for username_password"
        );

        let stored: UserAuthorityParams = user_authority
            .params
            .clone()
            .try_into()
            .expect("user authority params carry password_hash");
        assert!(
            stored
                .password_hash
                .starts_with("$argon2")
        );
        assert!(
            !stored
                .password_hash
                .contains("pass-word-1"),
            "neither the password nor the raw material may leak into the hash"
        );
        assert_eq!(
            verify_password(
                raw_password_hash("pass-word-1", PASSWORD_SALT, PASSWORD_PEPPER),
                stored.password_hash,
            ),
            Ok(true),
            "stored hash must verify against salt+pepper raw material"
        );
    }

    #[tokio::test]
    async fn rejects_password_and_confirmation_mismatch() {
        let strategy = username_password();

        let err = strategy
            .register(register_params(
                json!({ "password_confirmation": "different" }),
            ))
            .await
            .expect_err("mismatched confirmation must be rejected");

        assert_eq!(
            err.to_string(),
            "password and password confirmation do not match"
        );
    }

    #[tokio::test]
    async fn rejects_malformed_register_params() {
        let strategy = username_password();

        let err = strategy
            .register(JsonValue::new(json!({
                "username": "no-password",
                "password_confirmation": "whatever",
            })))
            .await
            .expect_err("missing password field must error");
        assert!(
            err.to_string()
                .contains("missing field `password`"),
            "expected serde missing-field error, got: {err}"
        );
    }

    #[tokio::test]
    async fn honors_an_explicit_kind_selection() {
        let strategy = username_password();

        let (user, _) = strategy
            .register(register_params(json!({ "kind": "api" })))
            .await
            .expect("valid register params must succeed");

        assert!(
            matches!(user.kind, Some(UserKind::Api)),
            "explicit kind must be honored, got {:?}",
            user.kind
        );
    }

    #[tokio::test]
    async fn duplicate_detection_lives_at_the_insert_boundary() {
        // Intentional layering (OXA-000050, the OXA-000033 Option-B pattern):
        // the `Registrar` contract is params in / DTOs out and gives a strategy
        // no repository handle, so it cannot see other users — and a check here
        // would be a check-then-insert, a TOCTOU race two concurrent
        // registrations always lose. Uniqueness is arbitrated by the
        // `users_username_key` unique index at the insert boundary, which
        // translates the violation into `UserAlreadyExistsError`. Pinned so the
        // registrar stays pure and nobody grows a duplicate check here.
        let strategy = username_password();

        strategy
            .register(register_params(json!({})))
            .await
            .expect("first register succeeds");
        let dupe = strategy
            .register(register_params(json!({})))
            .await;

        assert!(
            dupe.is_ok(),
            "the index is the arbiter, so a duplicate must still pass the registrar: {dupe:?}"
        );
    }

    #[tokio::test]
    async fn new_builds_a_working_registrar_from_the_pepper_env_var() {
        let _pepper = EnvGuard::set("OXIDAUTH_USERNAME_PASSWORD_PEPPER", PASSWORD_PEPPER);

        let authority = authority(authority_params_json());
        let registrar = new(&authority)
            .await
            .expect("registrar builds with the pepper env var set");

        let (user, user_authority) = registrar
            .register(register_params(json!({})))
            .await
            .expect("env-built registrar must register");
        assert_eq!(user.username, "new-user");

        let stored: UserAuthorityParams = user_authority
            .params
            .try_into()
            .expect("params carry password_hash");
        assert_eq!(
            verify_password(
                raw_password_hash("pass-word-1", PASSWORD_SALT, PASSWORD_PEPPER),
                stored.password_hash,
            ),
            Ok(true),
            "env-built registrar must hash with the env pepper"
        );
    }

    #[tokio::test]
    async fn new_errors_when_the_pepper_env_var_is_unset() {
        let _pepper = EnvGuard::unset("OXIDAUTH_USERNAME_PASSWORD_PEPPER");

        let res = new(&authority(authority_params_json())).await;
        match res {
            Ok(_) => panic!("new must fail when the pepper env var is unset"),
            Err(err) => {
                assert!(
                    err.to_string()
                        .contains("environment variable not found"),
                    "expected the env-var error, got: {err}"
                )
            },
        }
    }
}
