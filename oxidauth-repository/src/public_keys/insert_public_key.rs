use oxidauth_kernel::public_keys::PublicKey;

use crate::prelude::*;

#[async_trait]
pub trait InsertPublicKey: Send + Sync + 'static {
    async fn insert_public_key(
        &self,
        params: &InsertPublicKeyParams,
    ) -> Result<PublicKey, InsertPublicKeyError>;
}

#[derive(Debug)]
pub struct InsertPublicKeyParams {
    pub id: Option<Uuid>,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

#[derive(Debug)]
pub struct InsertPublicKeyError {}
