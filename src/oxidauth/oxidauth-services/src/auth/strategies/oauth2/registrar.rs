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

use super::{AuthorityParams, OAuth2};

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

        let user_authority = CreateUserAuthority {
            authority_id: self.authority_id,
            user_identifier: user.username.clone(),
            params: JsonValue::empty(),
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
}

impl Oauth2RegisterParams {
    pub fn to_value(&self) -> Result<Value, BoxedError> {
        Ok(serde_json::to_value(self)?)
    }
}

impl TryFrom<JsonValue> for Oauth2RegisterParams {
    type Error = BoxedError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        let params = serde_json::from_value(value.inner_value())?;

        Ok(params)
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

#[cfg(test)]
mod tests {
    use oxidauth_kernel::users::UserKind;
    use serde_json::json;

    use super::{
        super::{
            OAuthFlavors,
            fixtures::{AUTHORITY_ID, oauth2},
        },
        *,
    };

    #[tokio::test]
    async fn registers_a_passwordless_user_keyed_by_username() {
        let strategy = oauth2(OAuthFlavors::Google);

        let (user, user_authority) = strategy
            .register(JsonValue::new(json!({
                "username": "oauth-user",
                "email": "oauth@example.com",
                "first_name": "OAuth",
                "last_name": "User",
            })))
            .await
            .expect("valid oauth2 register params must succeed");

        assert_eq!(user.username, "oauth-user");
        assert_eq!(user.email.as_deref(), Some("oauth@example.com"));
        assert_eq!(user.first_name.as_deref(), Some("OAuth"));
        assert_eq!(user.last_name.as_deref(), Some("User"));
        assert!(user.id.is_some());
        assert!(matches!(user.kind, Some(UserKind::Human)));
        assert!(matches!(user.status, Some(UserStatus::Enabled)));

        assert_eq!(user_authority.authority_id, AUTHORITY_ID);
        // pinned: `register` keys the user authority by USERNAME, while
        // `user_authority_from_request` (the actual oauth2 callback path) keys
        // by EMAIL — two different identifier conventions coexist
        assert_eq!(user_authority.user_identifier, "oauth-user");
        assert!(
            user_authority
                .params
                .is_null(),
            "pinned: no secret or profile is stored in the user authority params",
        );
    }

    #[tokio::test]
    async fn honors_an_explicit_kind_selection() {
        let strategy = oauth2(OAuthFlavors::Microsoft);

        let (user, _) = strategy
            .register(JsonValue::new(json!({
                "username": "svc",
                "kind": "api",
            })))
            .await
            .expect("valid oauth2 register params must succeed");

        assert!(matches!(user.kind, Some(UserKind::Api)));
    }

    #[tokio::test]
    async fn rejects_malformed_register_params() {
        let strategy = oauth2(OAuthFlavors::Google);

        let err = strategy
            .register(JsonValue::new(
                json!({ "email": "no-username@example.com" }),
            ))
            .await
            .expect_err("missing username must error");
        assert!(
            err.to_string()
                .contains("missing field `username`"),
            "expected serde missing-field error, got: {err}"
        );

        assert!(
            strategy
                .register(JsonValue::new(json!("nope")))
                .await
                .is_err(),
            "non-object params must error"
        );
    }
}
