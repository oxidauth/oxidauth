use std::{fmt, sync::Arc};

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
pub trait AuthenticateServiceTrait: Send + Sync + 'static {
    async fn authenticate(
        &self,
        params: &AuthenticateParams,
    ) -> Result<AuthenticateResponse, BoxedError>;
}

pub type AuthenticateService = Arc<dyn AuthenticateServiceTrait>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticateParams {
    pub client_key: Uuid,
    pub params: JsonValue,
    /// `serde(skip)` on purpose: wire input can only ever produce
    /// `AuthEntry::External` (OXA-000004 step 1). In-process callers that
    /// legitimately drive another strategy (e.g. the oauth2 callback flow)
    /// set `AuthEntry::Internal` at construction.
    #[serde(skip)]
    pub entry: AuthEntry,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticateResponse {
    pub jwt: String,
    pub refresh_token: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug)]
pub enum ParseAuthorityStrategyError {
    Unknown(String),
}

impl fmt::Display for ParseAuthorityStrategyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ParseAuthorityStrategyError::*;

        match self {
            Unknown(value) => write!(f, "unknown authority strategy: {}", value),
        }
    }
}

impl From<&AuthenticateParams> for FindAuthorityByClientKey {
    fn from(value: &AuthenticateParams) -> Self {
        Self {
            client_key: value.client_key,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookReq {
    pub webhook_key: String,
    pub name: Option<String>,
    pub email: String,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookRes {
    pub success: bool,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn entry_is_never_readable_or_writable_through_the_wire_shape() {
        // OXA-000004 step 1: `entry` is `serde(skip)`ped — a request body
        // claiming `entry: "internal"` deserializes to `External`, and the
        // field never appears in serialized output either.
        let params: AuthenticateParams = serde_json::from_value(json!({
            "client_key": "7f92aedd-d5db-4f8e-8162-9baaf1ce0f05",
            "params": { "email": "someone@example.com" },
            "entry": "internal",
        }))
        .expect("wire body parses");

        assert_eq!(params.entry, AuthEntry::External);

        let value = serde_json::to_value(&params).expect("serializes");
        assert!(
            value.get("entry").is_none(),
            "entry must not round-trip: {value}"
        );
    }
}
