use crate::prelude::*;

use super::UserAuthorityRow;

#[async_trait]
pub trait UpdateUserAuthority: Send + Sync + 'static {
    async fn update_user_authoritiy_by_user_id(
        &self,
        params: &UpdateUserAuthorityParams,
    ) -> Result<UserAuthorityRow, UpdateUserAuthorityError>;
}

#[derive(Debug)]
pub struct UpdateUserAuthorityParams {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub user_identifier: String,
    pub params: serde_json::Value,
}

#[derive(Debug)]
pub struct UpdateUserAuthorityError {}
