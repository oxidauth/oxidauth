use crate::prelude::*;

pub use super::UserAuthorityRow;

#[async_trait]
pub trait DeleteUserAuthorityById: Send + Sync + 'static {
    async fn delete_user_authority_by_id(
        &self,
        user_authority_id: Uuid,
    ) -> Result<UserAuthorityRow, DeleteUserAuthorityByIdError>;
}

#[derive(Debug)]
pub struct DeleteUserAuthorityByIdError {}
