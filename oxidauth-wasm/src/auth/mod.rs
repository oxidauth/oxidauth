pub mod authenticate;
pub mod oauth2;
pub mod register;
pub mod username_password;

use crate::BoxedError;
pub use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

#[cfg(feature = "mock")]
use super::mock::ClientMock;

use super::{Client, Resource, fmt, handle_response};
pub use crate::auth::{authenticate::AuthenticateTrait, register::RegisterTrait};

#[derive(Debug, Serialize, Deserialize)]
pub struct Oauth2RedirectRes {
    pub redirect_url: Url,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRes {
    pub jwt: String,
    pub refresh_token: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticateReq {
    pub client_key: Uuid,
    pub params: serde_json::Value,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityStrategy {
    UsernamePassword,
    SingleUseToken,
    Oauth2,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticateRes {
    pub jwt: String,
    pub refresh_token: Uuid,
}

pub trait AuthTrait: RegisterTrait + AuthenticateTrait {}

impl AuthTrait for Client {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterParams {
    pub client_key: Uuid,
    pub params: serde_json::Value,
}

#[cfg(feature = "mock")]
impl AuthTrait for ClientMock {}
