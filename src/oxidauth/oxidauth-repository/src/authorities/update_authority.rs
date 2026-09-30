pub use oxidauth_kernel::authorities::{Authority, update_authority::UpdateAuthority};

pub use crate::prelude::*;

#[async_trait]
pub trait UpdateAuthorityQuery: Send + Sync + 'static {
    async fn update_authority(&self, params: &UpdateAuthority) -> Result<Authority, BoxedError>;
}
