use crate::prelude::*;

pub use super::AuthorityRow;

#[async_trait]
pub trait UpdateAuthority: Send + Sync + 'static {
    async fn update_authority(
        &self,
        params: &UpdateAuthorityParams,
    ) -> Result<AuthorityRow, UpdateAuthorityError>;
}

#[derive(Debug)]
pub struct UpdateAuthorityParams {
    pub id: Uuid,
    pub name: String,
    pub client_key: Uuid,
    pub status: String,
    pub strategy: String,
    pub settings: serde_json::Value,
    pub params: serde_json::Value,
}

#[derive(Debug)]
pub struct UpdateAuthorityError {}
