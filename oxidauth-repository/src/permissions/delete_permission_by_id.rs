pub use oxidauth_kernel::permissions::Permission;

use crate::prelude::*;

#[async_trait]
pub trait DeletePermissionById: Send + Sync + 'static {
    async fn delete_permission_by_id(
        &self,
        permission_id: Uuid,
    ) -> Result<Permission, DeletePermissionByIdError>;
}

#[derive(Debug)]
pub struct DeletePermissionByIdError {}
