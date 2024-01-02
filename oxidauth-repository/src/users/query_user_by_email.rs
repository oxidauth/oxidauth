pub use oxidauth_kernel::users::User;

use crate::prelude::*;

#[async_trait]
pub trait QueryUserByEmail: Send + Sync + 'static {
    async fn query_user_by_email(
        &self,
        email: String,
    ) -> Result<User, QueryUserByEmailError>;
}

#[derive(Debug)]
pub struct QueryUserByEmailError {}
