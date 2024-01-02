use crate::prelude::*;

use super::AuthorityRow;

#[async_trait]
pub trait QueryAllAuthorities: Send + Sync + 'static {
    async fn query_all_authorities(
        &self,
    ) -> Result<AuthorityRow, QueryAllAuthoritiesError>;
}

#[derive(Debug)]
pub struct QueryAllAuthoritiesError {}
