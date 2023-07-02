use crate::prelude::*;

pub use super::UserAuthorityRow;

#[async_trait]
pub trait UpdateUserAuthorityByUserId: Send + Sync + 'static {
    async fn update_user_authority_by_user_id(
        &self,
        params: &UpdateUserAuthorityByUserIdParams,
    ) -> Result<UserAuthorityRow, UpdateUserAuthorityError>;
}

#[derive(Debug)]
pub struct UpdateUserAuthorityByUserIdParams {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub user_identifier: String,
    pub params: serde_json::Value,
}

#[derive(Debug)]
pub struct UpdateUserAuthorityError {}
