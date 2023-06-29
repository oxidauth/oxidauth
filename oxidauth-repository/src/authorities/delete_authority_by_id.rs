use crate::prelude::*;

use super::AuthorityRow;

#[async_trait]
pub trait DeleteAuthorityById: Send + Sync + 'static {
    async fn delete_authority_by_id(
        &self,
        authority_id: Uuid,
    ) -> Result<AuthorityRow, DeleteAuthorityByIdError>;
}

#[derive(Debug)]
pub struct DeleteAuthorityByIdError {}
