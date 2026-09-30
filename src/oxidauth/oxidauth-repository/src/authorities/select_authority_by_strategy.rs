pub use oxidauth_kernel::authorities::{
    Authority,
    find_authority_by_strategy::FindAuthorityByStrategy,
};

pub use crate::prelude::*;

#[async_trait]
pub trait SelectAuthorityByStrategyQuery: Send + Sync + 'static {
    async fn select_authority_by_strategy(
        &self,
        params: &FindAuthorityByStrategy,
    ) -> Result<Option<Authority>, BoxedError>;
}
