pub use oxidauth_kernel::users::find_users_by_ids::FindUsersByIds;
use oxidauth_kernel::users::find_users_by_ids::UsersByIds;

use crate::prelude::*;

#[async_trait]
pub trait SelectUsersByIdsQuery: Send + Sync + 'static {
    async fn select_users_by_ids(&self, params: &FindUsersByIds) -> Result<UsersByIds, BoxedError>;
}
