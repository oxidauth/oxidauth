pub use oxidauth_kernel::users::{User, find_user_by_id::FindUserById};

use crate::prelude::*;

#[async_trait]
pub trait SelectUserByIdQuery: Send + Sync + 'static {
    async fn select_user_by_id(&self, params: &FindUserById) -> Result<User, BoxedError>;
}
