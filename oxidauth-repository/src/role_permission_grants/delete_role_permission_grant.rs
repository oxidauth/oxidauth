use crate::prelude::*;

pub use super::RolePermissionGrantRow;

#[async_trait]
pub trait DeleteRolePermissionGrant: Send + Sync + 'static {
    async fn delete_role_permission_grant(
        &self,
        params: &DeleteRolePermissionGrantParams,
    ) -> Result<RolePermissionGrantRow, DeleteRolePermissionGrantError>;
}

#[derive(Debug)]
pub struct DeleteRolePermissionGrantParams {
    pub role_id: Uuid,
    pub permission_id: Uuid,
}

#[derive(Debug)]
pub struct DeleteRolePermissionGrantError {}
