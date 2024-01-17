pub use oxidauth_kernel::role_permission_grants::RolePermissionGrant;

use crate::prelude::*;

#[async_trait]
pub trait DeleteRolePermissionGrant: Send + Sync + 'static {
    async fn delete_role_permission_grant(
        &self,
        params: &DeleteRolePermissionGrantParams,
    ) -> Result<RolePermissionGrant, DeleteRolePermissionGrantError>;
}

#[derive(Debug)]
pub struct DeleteRolePermissionGrantParams {
    pub role_id: Uuid,
    pub permission_id: Uuid,
}

#[derive(Debug)]
pub struct DeleteRolePermissionGrantError {}
