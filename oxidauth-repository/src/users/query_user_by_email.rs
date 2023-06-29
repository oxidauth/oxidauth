use crate::prelude::*;

use super::UserRow;

#[async_trait]
pub trait QueryUserByEmail: Send + Sync + 'static {
    async fn query_user_by_email(&self, email: String) -> Result<UserRow, QueryUserByEmailError>;
}

#[derive(Debug)]
pub struct QueryUserByEmailError {}
