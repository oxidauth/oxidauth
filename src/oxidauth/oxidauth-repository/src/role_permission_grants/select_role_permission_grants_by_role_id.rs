use oxidauth_kernel::role_permission_grants::list_role_permission_grants_by_role_id::{
    ListRolePermissionGrantsByRoleId,
    *,
};

pub use crate::prelude::*;

#[async_trait]
pub trait SelectRolePermissionGrantsByRoleIdQuery: Send + Sync + 'static {
    async fn select_role_permission_grants_by_role_id(
        &self,
        params: &ListRolePermissionGrantsByRoleId,
    ) -> Result<Vec<RolePermission>, BoxedError>;
}
