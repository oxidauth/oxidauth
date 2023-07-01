use crate::prelude::*;

pub use super::PublicKeyRow;

#[async_trait]
pub trait DeletePublicKeyById: Send + Sync + 'static {
    async fn delete_public_key_by_id(
        &self,
        public_key_id: Uuid,
    ) -> Result<PublicKeyRow, DeletePublicKeyByIdError>;
}

#[derive(Debug)]
pub struct DeletePublicKeyByIdError {}
