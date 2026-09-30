pub use oxidauth_kernel::authorities::{Authority, create_authority::CreateAuthority};

pub use crate::prelude::*;

#[async_trait]
pub trait InsertAuthorityQuery: Send + Sync + 'static {
    async fn insert_authority(&self, params: &CreateAuthority) -> Result<Authority, BoxedError>;
}
