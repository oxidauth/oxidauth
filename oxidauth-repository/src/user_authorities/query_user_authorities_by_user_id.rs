use crate::prelude::*;

use super::UserAuthorityRow;

#[async_trait]
pub trait QueryUserAuthoritiesByUserId: Send + Sync + 'static {
    async fn query_user_authorities_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserAuthorityRow>, QueryUserAuthoritiesByUserIdError>;
}

#[derive(Debug)]
pub struct QueryUserAuthoritiesByUserIdError {}
