use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use super::{AuthEntry, authenticate::AuthenticateParams, register::RegisterParams};
use crate::{JsonValue, dev_prelude::BoxedError};

#[async_trait]
pub trait AuthenticateOrRegisterServiceTrait: Send + Sync + 'static {
    async fn authenticate_or_register(
        &self,
        params: &AuthenticateOrRegisterParams,
    ) -> Result<AuthenticateOrRegisterResponse, BoxedError>;
}

pub type AuthenticateOrRegisterService = Arc<dyn AuthenticateOrRegisterServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticateOrRegisterResponse {
    pub jwt: String,
    pub refresh_token: Uuid,
    pub client_base: Url,
    pub email: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OAuth2AuthenticateParams {
    pub code: String,
    pub scope: Option<String>,
    pub client_key: Uuid,
}

impl TryFrom<JsonValue> for OAuth2AuthenticateParams {
    type Error = BoxedError;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        let params = serde_json::from_value(value.inner_value())?;

        Ok(params)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticateOrRegisterParams {
    pub client_key: Uuid,
    pub state: String,
    pub params: JsonValue,
}

// The oauth2 callback flow is the only producer of these params; its hops
// into the generic pipeline are in-process and follow a completed IdP code
// exchange, hence `Internal` (OXA-000004 step 1).
impl From<&AuthenticateOrRegisterParams> for AuthenticateParams {
    fn from(value: &AuthenticateOrRegisterParams) -> Self {
        Self {
            client_key: value.client_key,
            params: value.params.clone(),
            entry: AuthEntry::Internal,
        }
    }
}

impl From<&AuthenticateOrRegisterParams> for RegisterParams {
    fn from(value: &AuthenticateOrRegisterParams) -> Self {
        Self {
            client_key: value.client_key,
            params: value.params.clone(),
            entry: AuthEntry::Internal,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OAuth2AuthenticatePathParams {
    pub code: String,
    pub scope: Option<String>,
    pub state: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OAuth2Profile {
    pub email: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
}
