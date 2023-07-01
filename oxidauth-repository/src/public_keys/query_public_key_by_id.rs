use crate::prelude::*;

pub use super::PublicKeyRow;

#[async_trait]
pub trait QueryPublicKeyById: Send + Sync + 'static {
    async fn query_public_key_by_id(
        &self,
        public_key_id: Uuid,
    ) -> Result<PublicKeyRow, QueryPublicKeyByIdError>;
}

#[derive(Debug)]
pub struct QueryPublicKeyByIdError {}
