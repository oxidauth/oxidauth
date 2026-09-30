pub use oxidauth_kernel::user_role_grants::{
    UserRole,
    list_user_role_grants_by_user_id::ListUserRoleGrantsByUserId,
};

use crate::prelude::*;

#[async_trait]
pub trait SelectUserRoleGrantsByUserIdQuery: Send + Sync + 'static {
    async fn select_user_role_grants_by_user_id(
        &self,
        params: &ListUserRoleGrantsByUserId,
    ) -> Result<Vec<UserRole>, BoxedError>;
}
