use crate::prelude::*;

use super::PermissionRow;

#[async_trait]
pub trait DeletePermission: Send + Sync + 'static {
    async fn delete_permission(
        &self,
        permission_id: Uuid,
    ) -> Result<PermissionRow, DeletePermissionError>;
}

#[derive(Debug)]
pub struct DeletePermissionError {}
