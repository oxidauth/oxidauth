pub use oxidauth_kernel::authorities::{Authority, delete_authority::DeleteAuthority};

pub use crate::prelude::*;

#[async_trait]
pub trait DeleteAuthorityQuery: Send + Sync + 'static {
    async fn delete_authority(&self, params: &DeleteAuthority) -> Result<Authority, BoxedError>;
}
