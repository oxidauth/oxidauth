use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    JsonValue,
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
