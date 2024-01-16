use oxidauth_kernel::user_permission_grants::UserPermissionGrant;

use crate::prelude::*;

#[async_trait]
pub trait DeleteUserPermissionGrant: Send + Sync + 'static {
    async fn delete_user_permission_grant(
        &self,
        params: &DeleteUserPermissionGrantParams,
    ) -> Result<UserPermissionGrant, DeleteUserPermissionGrantError>;
}

#[derive(Debug)]
pub struct DeleteUserPermissionGrantParams {
    pub user_id: Uuid,
    pub permission_id: Uuid,
}

#[derive(Debug)]
pub struct DeleteUserPermissionGrantError {}
