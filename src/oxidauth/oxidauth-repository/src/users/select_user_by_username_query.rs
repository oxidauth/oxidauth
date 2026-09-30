pub use oxidauth_kernel::users::{User, Username};

use crate::prelude::*;

#[async_trait]
pub trait SelectUserByUsernameQuery: Send + Sync + 'static {
    async fn select_user_by_username(&self, params: &Username) -> Result<Option<User>, BoxedError>;
}
