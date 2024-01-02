pub use oxidauth_kernel::users::User;

use crate::prelude::*;

#[async_trait]
pub trait QueryUserById: Send + Sync + 'static {
    async fn query_user_by_id(
        &self,
        user_id: Uuid,
    ) -> Result<User, QueryUserByIdError>;
}

#[derive(Debug)]
pub struct QueryUserByIdError {}
