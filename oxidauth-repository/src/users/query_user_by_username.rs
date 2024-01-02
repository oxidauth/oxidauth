pub use oxidauth_kernel::users::User;

use crate::prelude::*;

#[async_trait]
pub trait QueryUserByUsername: Send + Sync + 'static {
    async fn query_user_by_username(
        &self,
        username: String,
    ) -> Result<User, QueryUserByUsernameError>;
}

#[derive(Debug)]
pub struct QueryUserByUsernameError {}
