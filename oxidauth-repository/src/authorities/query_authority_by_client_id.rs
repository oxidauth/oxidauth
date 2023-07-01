use crate::prelude::*;

pub use super::AuthorityRow;

#[async_trait]
pub trait QueryAuthorityByClientId: Send + Sync + 'static {
    async fn query_authority_by_client_id(
        &self,
        client_id: Uuid,
    ) -> Result<AuthorityRow, QueryAuthorityByClientIdError>;
}

#[derive(Debug)]
pub struct QueryAuthorityByClientIdError {}
