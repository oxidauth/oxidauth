pub use oxidauth_kernel::users::User;
use oxidauth_kernel::users::create_user::CreateUser;

pub use crate::prelude::*;

#[async_trait]
pub trait InsertUserQuery: Send + Sync + 'static {
    async fn insert_user(&self, params: &CreateUser) -> Result<User, BoxedError>;
}
