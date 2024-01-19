use oxidauth_kernel::public_keys::PublicKey;

use crate::prelude::*;

#[async_trait]
pub trait QueryPublicKeyById: Send + Sync + 'static {
    async fn query_public_key_by_id(
        &self,
        public_key_id: Uuid,
    ) -> Result<PublicKey, QueryPublicKeyByIdError>;
}

#[derive(Debug)]
pub struct QueryPublicKeyByIdError {}
