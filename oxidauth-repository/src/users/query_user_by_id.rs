use crate::prelude::*;

use super::UserRow;

#[async_trait]
pub trait QueryUserById: Send + Sync + 'static {
    async fn query_user_by_id(&self, user_id: Uuid) -> Result<UserRow, QueryUserByIdError>;
}

#[derive(Debug)]
pub struct QueryUserByIdError {}
