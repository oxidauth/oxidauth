use crate::prelude::*;

pub use super::UserAuthorityRow;

#[async_trait]
pub trait DeleteUserAuthorityById: Send + Sync + 'static {
    async fn delete_user_authority_by_id(
        &self,
        user_authority_id: Uuid,
    ) -> Result<UserAuthorityRow, DeleteUserAuthorityByIdError>;
}

// @GEORGE - struggling here, this is delete user by id but table has no id - should it be combo of user id and authority id?

#[derive(Debug)]
pub struct DeleteUserAuthorityByIdError {}
