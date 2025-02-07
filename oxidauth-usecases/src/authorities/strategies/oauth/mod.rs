use serde::{Deserialize, Serialize};

use super::{
    AuthenticateStrategy, AuthenticateStrategyError, RegisterStrategy,
    RegisterStrategyError,
};

pub mod authenticate;
pub mod find_redirect_url_by_authority_client_key;
pub mod register;

pub struct OauthStrategy {}
