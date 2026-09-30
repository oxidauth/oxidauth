pub use oxidauth_kernel::authorities::{Authority, find_authority_by_id::FindAuthorityById};

pub use crate::prelude::*;

#[async_trait]
pub trait SelectAuthorityByIdQuery: Send + Sync + 'static {
    async fn select_authority_by_id(
        &self,
        params: &FindAuthorityById,
    ) -> Result<Authority, BoxedError>;
}
