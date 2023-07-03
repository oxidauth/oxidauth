use serde::{Deserialize, Serialize};

use super::{
    AuthenticateStrategy, AuthenticateStrategyError, RegisterStrategy, RegisterStrategyError,
};

pub mod authenticate;
pub mod register;

pub struct UsernamePasswordStrategy {}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsernamePasswordAuthorityParams {
    pub pepper: String,
    pub pepper_env_var_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsernamePasswordUserAuthorityParams {
    pub password_hash: String,
}
