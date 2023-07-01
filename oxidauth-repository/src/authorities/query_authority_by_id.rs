use crate::prelude::*;

pub use super::AuthorityRow;

#[async_trait]
pub trait QueryAuthorityById: Send + Sync + 'static {
    async fn query_authority_by_id(
        &self,
        authority_id: Uuid,
    ) -> Result<AuthorityRow, QueryAuthorityByIdError>;
}

#[derive(Debug)]
pub struct QueryAuthorityByIdError {}
