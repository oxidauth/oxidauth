use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    authorities::find_authority_by_client_key::FindAuthorityByClientKey,
    dev_prelude::{BoxedError, Service},
    JsonValue,
};

pub type RegisterService = Arc<
    dyn for<'a> Service<
        &'a RegisterParams,
        Response = RegisterResponse,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterParams {
    pub client_key: Uuid,
    pub params: JsonValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub jwt: String,
    pub refresh_token: Uuid,
}

impl From<&RegisterParams> for FindAuthorityByClientKey {
    fn from(value: &RegisterParams) -> Self {
        Self {
            client_key: value.client_key,
        }
    }
}
