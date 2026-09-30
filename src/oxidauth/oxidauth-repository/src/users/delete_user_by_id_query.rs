pub use oxidauth_kernel::users::{User, delete_user_by_id::DeleteUserById};

use crate::prelude::*;

#[async_trait]
pub trait DeleteUserByIdQuery: Send + Sync + 'static {
    async fn delete_user_by_id(&self, params: &DeleteUserById) -> Result<User, BoxedError>;
}
