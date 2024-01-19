use oxidauth_kernel::public_keys::PublicKey;

use crate::prelude::*;

#[async_trait]
pub trait DeletePublicKeyById: Send + Sync + 'static {
    async fn delete_public_key_by_id(
        &self,
        public_key_id: Uuid,
    ) -> Result<PublicKey, DeletePublicKeyByIdError>;
}

#[derive(Debug)]
pub struct DeletePublicKeyByIdError {}
