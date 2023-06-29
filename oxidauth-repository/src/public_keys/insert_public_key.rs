use crate::prelude::*;

use super::PublicKeyRow;

#[async_trait]
pub trait InsertPublicKey: Send + Sync + 'static {
    async fn insert_public_key(
        &self,
        params: InsertPublicKeyParams,
    ) -> Result<PublicKeyRow, InsertPublicKeyError>;
}

#[derive(Debug)]
pub struct InsertPublicKeyParams {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

#[derive(Debug)]
pub struct InsertPublicKeyError {}
