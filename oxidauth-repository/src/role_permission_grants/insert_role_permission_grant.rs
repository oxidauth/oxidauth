use crate::prelude::*;

pub use super::RolePermissionGrantRow;

#[async_trait]
pub trait InsertRolePermissionGrant: Send + Sync + 'static {
    async fn insert_role_permission_grant(
        &self,
        params: &InsertRolePermissionGrantParams,
    ) -> Result<RolePermissionGrantRow, InsertRolePermissionGrantError>;
}

#[derive(Debug)]
pub struct InsertRolePermissionGrantParams {
    pub role_id: Uuid,
    pub permission_id: Uuid,
}

#[derive(Debug)]
pub struct InsertRolePermissionGrantError {}
