use async_trait::async_trait;
use oxidauth_kernel::{
    JsonValue,
    auth::Registrar,
    authorities::Authority,
    error::BoxedError,
    user_authorities::create_user_authority::CreateUserAuthority,
    users::{UserKind, UserStatus, create_user::CreateUser},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::{AuthorityParams, OAuth2, OAuth2AuthorityParams};

#[async_trait]
impl Registrar for OAuth2 {
    #[tracing::instrument(name = "oauth2 register", skip(self))]
    async fn register(
        &self,
        register_params: JsonValue,
    ) -> Result<(CreateUser, CreateUserAuthority), BoxedError> {
        let register_params: Oauth2RegisterParams = register_params
            .clone()
            .try_into()?;

        let user: CreateUser = register_params.clone().into();

        let params = OAuth2AuthorityParams {
            access_token: register_params.access_token,
        };

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

    Ok(Box::new(OAuth2 {
        authority_id,
        params,
    }))
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Oauth2RegisterParams {
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub kind: Option<UserKind>,
    pub last_name: Option<String>,
    pub username: String,
    pub access_token: String,
}

impl Oauth2RegisterParams {
    pub fn to_value(&self) -> Result<Value, BoxedError> {
        Ok(serde_json::to_value(self)?)
    }
}

impl TryFrom<JsonValue> for Oauth2RegisterParams {
    type Error = BoxedError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        let s: Self = serde_json::from_value(value.inner_value())?;

        Ok(s)
    }
}

impl From<Oauth2RegisterParams> for CreateUser {
    fn from(params: Oauth2RegisterParams) -> Self {
        let Oauth2RegisterParams {
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
