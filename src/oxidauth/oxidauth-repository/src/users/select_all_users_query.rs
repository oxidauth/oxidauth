pub use oxidauth_kernel::users::{User, list_all_users::ListAllUsers};

use crate::prelude::*;

#[async_trait]
pub trait SelectAllUsersQuery: Send + Sync + 'static {
    async fn select_all_users(&self, params: &ListAllUsers) -> Result<Vec<User>, BoxedError>;
}
