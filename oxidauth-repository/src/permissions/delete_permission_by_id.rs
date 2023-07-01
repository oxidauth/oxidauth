use crate::prelude::*;

pub use super::PermissionRow;

#[async_trait]
pub trait DeletePermissionById: Send + Sync + 'static {
    async fn delete_permission_by_id(
        &self,
        permission_id: Uuid,
    ) -> Result<PermissionRow, DeletePermissionByIdError>;
}

#[derive(Debug)]
pub struct DeletePermissionByIdError {}
