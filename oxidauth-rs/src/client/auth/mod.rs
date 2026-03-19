pub mod authenticate;
pub mod oauth2;
pub mod register;
pub mod username_password;

#[cfg(feature = "mock")]
use super::mock::ClientMock;
use super::{
    Client,
    Resource,
    fmt,
    handle_response,
};
pub use crate::auth::{
    authenticate::AuthenticateTrait,
    register::RegisterTrait,
};

pub trait AuthTrait: RegisterTrait + AuthenticateTrait {}

impl AuthTrait for Client {
}

#[cfg(feature = "mock")]
impl AuthTrait for ClientMock {
}
