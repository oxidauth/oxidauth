use crate::dev_prelude::*;

use super::Authority;

#[async_trait]
pub trait FindAuthorityByClientIdService: Send + Sync + 'static {
    async fn find_authority_by_client_id(
        &self,
        authority_id: Uuid,
    ) -> Result<Authority, FindAuthorityByClientIdError>;
}

#[derive(Debug)]
pub struct FindAuthorityByClientIdError {}
