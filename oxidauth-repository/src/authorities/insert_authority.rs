use crate::prelude::*;

pub use super::AuthorityRow;

#[async_trait]
pub trait InsertAuthority: Send + Sync + 'static {
    async fn insert_authority(
        &self,
        params: InsertAuthorityParams,
    ) -> Result<AuthorityRow, InsertAuthorityError>;
}

#[derive(Debug)]
pub struct InsertAuthorityParams {
    pub name: String,
    pub client_key: Uuid,
    pub status: String,
    pub strategy: String,
    pub settings: serde_json::Value,
    pub params: serde_json::Value,
}

#[derive(Debug)]
pub struct InsertAuthorityError {}
