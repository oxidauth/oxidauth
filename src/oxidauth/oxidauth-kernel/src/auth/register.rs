use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    JsonValue,
    auth::AuthEntry,
    authorities::find_authority_by_client_key::FindAuthorityByClientKey,
    dev_prelude::BoxedError,
};

#[async_trait]
pub trait RegisterServiceTrait: Send + Sync + 'static {
    async fn register(&self, params: &RegisterParams) -> Result<RegisterResponse, BoxedError>;
}

pub type RegisterService = Arc<dyn RegisterServiceTrait>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterParams {
    pub client_key: Uuid,
    pub params: JsonValue,
    /// `serde(skip)` on purpose: wire input can only ever produce
    /// `AuthEntry::External` (OXA-000004 step 1). In-process callers that
    /// legitimately drive another strategy (e.g. the oauth2 callback flow's
    /// register fallback) set `AuthEntry::Internal` at construction.
    #[serde(skip)]
    pub entry: AuthEntry,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub jwt: String,
    pub refresh_token: Uuid,
    pub user_id: Uuid,
}

impl From<&RegisterParams> for FindAuthorityByClientKey {
    fn from(value: &RegisterParams) -> Self {
        Self {
            client_key: value.client_key,
        }
    }
}
