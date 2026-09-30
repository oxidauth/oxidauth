pub use oxidauth_kernel::authorities::{Authority, list_all_authorities::ListAllAuthorities};

pub use crate::prelude::*;

#[async_trait]
pub trait SelectAllAuthoritiesQuery: Send + Sync + 'static {
    async fn select_all_authorities(
        &self,
        params: &ListAllAuthorities,
    ) -> Result<Vec<Authority>, BoxedError>;
}
