use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::BoxedError;

pub use super::{Authority, AuthorityStrategy};

#[async_trait]
pub trait FindAuthorityByStrategyTrait: Send + Sync + 'static {
    async fn find_authority_by_strategy(
        &self,
        params: &FindAuthorityByStrategy,
    ) -> Result<Authority, BoxedError>;
}

pub type FindAuthorityByStrategyService = Arc<dyn FindAuthorityByStrategyTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindAuthorityByStrategy {
    pub strategy: AuthorityStrategy,
}
