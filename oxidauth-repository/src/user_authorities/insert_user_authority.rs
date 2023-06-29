use crate::prelude::*;

use super::UserAuthorityRow;

#[async_trait]
pub trait InsertUserAuthority: Send + Sync + 'static {
    async fn insert_user_authority(
        &self,
        insert_user_authority: &InsertUserAuthorityParams,
    ) -> Result<UserAuthorityRow, InsertUserAuthorityError>;
}

#[derive(Debug)]
pub struct InsertUserAuthorityParams {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub user_identifier: String,
    pub params: serde_json::Value,
}

#[derive(Debug)]
pub struct InsertUserAuthorityError {}
