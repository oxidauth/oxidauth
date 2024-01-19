use std::sync::Arc;

use serde::Deserialize;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::{Authority, AuthorityStrategy};

pub type FindAuthorityByStrategyService = Arc<
    dyn for<'a> Service<
        &'a FindAuthorityByStrategy,
        Response = Authority,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct FindAuthorityByStrategy {
    pub strategy: AuthorityStrategy,
}

