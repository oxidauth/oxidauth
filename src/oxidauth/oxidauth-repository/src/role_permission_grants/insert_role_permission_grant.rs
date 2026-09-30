use oxidauth_kernel::role_permission_grants::create_role_permission_grant::*;

pub use crate::prelude::*;

#[async_trait]
pub trait InsertRolePermissionGrantQuery: Send + Sync + 'static {
    async fn insert_role_permission_grant(
        &self,
        params: &InsertRolePermissionGrant,
    ) -> Result<RolePermissionGrant, BoxedError>;
}
#[derive(Debug)]
pub struct InsertRolePermissionGrant {
    pub role_id: Uuid,
    pub permission_id: Uuid,
}
