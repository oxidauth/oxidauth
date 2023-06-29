use crate::prelude::*;

use super::UserPermissionGrantRow;

#[async_trait]
pub trait DeleteUserPermissionGrant: Send + Sync + 'static {
    async fn delete_user_permission_grant(
        &self,
        params: DeleteUserPermissionGrantParams,
    ) -> Result<UserPermissionGrantRow, DeleteUserPermissionGrantError>;
}

#[derive(Debug)]
pub struct DeleteUserPermissionGrantParams {
    pub user_id: Uuid,
    pub permission_id: Uuid,
}

#[derive(Debug)]
pub struct DeleteUserPermissionGrantError {}
