use serde::{Deserialize, Serialize};

use super::{
    AuthenticateStrategy, AuthenticateStrategyError, RegisterStrategy,
    RegisterStrategyError,
};

pub mod authenticate;
pub mod redirect_url;
pub mod register;

pub struct Oauth2Strategy {}
