use serde::Serialize;

use crate::dev_prelude::*;

pub mod oauth;
pub mod username_password;

pub trait RegisterStrategy<P>: Send + Sync + 'static {
    type UserAuthorityParams: Serialize;

    fn user_authority_params(
        &self,
        authority_params: Value,
        params: P,
    ) -> Result<Self::UserAuthorityParams, RegisterStrategyError>;
}

#[derive(Debug)]
pub struct RegisterStrategyError {}

pub trait AuthenticateStrategy<P>: Send + Sync + 'static {
    type AuthorityParams;
    type UserAuthorityParams;

    fn authenticate(
        &self,
        authority_params: Self::AuthorityParams,
        user_authority_params: Self::UserAuthorityParams,
        params: P,
    ) -> Result<(), AuthenticateStrategyError>;
}

#[derive(Debug)]
pub struct AuthenticateStrategyError {}
