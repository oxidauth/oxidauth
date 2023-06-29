use crate::prelude::*;

use super::UserRow;

#[async_trait]
pub trait QueryUserByUsername: Send + Sync + 'static {
    async fn query_user_by_username(
        &self,
        username: String,
    ) -> Result<UserRow, QueryUserByUsernameError>;
}

#[derive(Debug)]
pub struct QueryUserByUsernameError {}
