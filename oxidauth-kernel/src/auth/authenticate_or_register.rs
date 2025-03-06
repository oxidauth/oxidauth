use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    JsonValue,
    dev_prelude::{BoxedError, Service},
};

use super::{
    authenticate::{AuthenticateParams, AuthenticateResponse},
    register::RegisterParams,
};

pub type AuthenticateOrRegisterService = Arc<
    dyn for<'a> Service<
            &'a AuthenticateOrRegisterParams,
            Response = AuthenticateResponse,
            Error = BoxedError,
        >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticateOrRegisterParams {
    pub client_key: Uuid,
    pub params: JsonValue,
}

impl From<&AuthenticateOrRegisterParams> for AuthenticateParams {
    fn from(value: &AuthenticateOrRegisterParams) -> Self {
        Self {
            client_key: value.client_key,
            params: value.params.clone(),
        }
    }
}

impl From<&AuthenticateOrRegisterParams> for RegisterParams {
    fn from(value: &AuthenticateOrRegisterParams) -> Self {
        Self {
            client_key: value.client_key,
            params: value.params.clone(),
        }
    }
}
