use std::sync::Arc;

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct FindAuthorityByStrategy {
    pub strategy: AuthorityStrategy,
}
