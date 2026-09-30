pub use oxidauth_kernel::user_permission_grants::{
    UserPermission,
    list_user_permission_grants_by_user_id::ListUserPermissionGrantsByUserId,
};

use crate::prelude::*;

#[async_trait]
pub trait SelectUserPermissionGrantsByUserIdQuery: Send + Sync + 'static {
    async fn select_user_permission_grants_by_user_id(
        &self,
        params: &ListUserPermissionGrantsByUserId,
    ) -> Result<Vec<UserPermission>, BoxedError>;
}
