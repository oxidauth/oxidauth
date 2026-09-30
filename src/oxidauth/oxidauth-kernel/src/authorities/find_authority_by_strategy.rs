use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use super::{Authority, AuthorityStrategy};
use crate::error::BoxedError;

#[async_trait]
pub trait FindAuthorityByStrategyServiceTrait: Send + Sync + 'static {
    async fn find_authority_by_strategy(
        &self,
        params: &FindAuthorityByStrategy,
    ) -> Result<Authority, BoxedError>;
}

pub type FindAuthorityByStrategyService = Arc<dyn FindAuthorityByStrategyServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindAuthorityByStrategy {
    pub strategy: AuthorityStrategy,
}
