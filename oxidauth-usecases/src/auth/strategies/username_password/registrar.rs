use async_trait::async_trait;
use oxidauth_kernel::{
    auth::Registrar,
    authorities::Authority,
    error::BoxedError,
    user_authorities::create_user_authority::CreateUserAuthority,
    users::{create_user::CreateUser, UserKind, UserStatus},
    JsonValue, Password,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::{
    helpers::{hash_password, raw_password_hash},
    AuthorityParams, UserAuthorityParams, UsernamePassword,
};

#[async_trait]
impl Registrar for UsernamePassword {
    #[tracing::instrument(name = "username_password register", skip(self))]
    async fn register(
        &self,
        register_params: JsonValue,
    ) -> Result<
        (
            CreateUser,
            CreateUserAuthority,
        ),
        BoxedError,
    > {
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
            return Err(
                "password and password confirmation do not match".into(),
            );
        }

        let user: CreateUser = register_params.clone().into();

        let password = raw_password_hash(
            &password,
            &self.params.password_salt,
            &self.password_pepper,
        );

        let password_hash =
            hash_password(password).map_err(|err| err.to_string())?;

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

pub async fn new(
    authority: &Authority,
) -> Result<Box<dyn Registrar>, BoxedError> {
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

#[derive(Clone, Serialize, Deserialize)]
pub struct UsernamePasswordRegisterParams {
    pub username: String,
    pub password: Password,
    pub password_confirmation: Password,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub kind: Option<UserKind>,
}

impl UsernamePasswordRegisterParams {
    pub fn to_value(&self) -> Result<Value, BoxedError> {
        Ok(serde_json::to_value(self)?)
    }
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
