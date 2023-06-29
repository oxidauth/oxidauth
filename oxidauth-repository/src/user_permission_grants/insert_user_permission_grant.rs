use crate::prelude::*;

use super::UserPermissionGrantRow;

#[async_trait]
pub trait InsertUserPermissionGrant: Send + Sync + 'static {
    async fn delete_user_permission_grant(
        &self,
        params: InsertUserPermissionGrantParams,
    ) -> Result<UserPermissionGrantRow, InsertUserPermissionGrantError>;
}

#[derive(Debug)]
pub struct InsertUserPermissionGrantParams {
    pub user_id: Uuid,
    pub permission_id: Uuid,
}

#[derive(Debug)]
pub struct InsertUserPermissionGrantError {}
