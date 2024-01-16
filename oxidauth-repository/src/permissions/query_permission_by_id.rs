pub use oxidauth_kernel::permissions::Permission;

use crate::prelude::*;

#[async_trait]
pub trait QueryPermissionById: Send + Sync + 'static {
    async fn query_permission_by_id(
        &self,
        permission_id: Uuid,
    ) -> Result<Permission, QueryPermissionByIdError>;
}

#[derive(Debug)]
pub struct QueryPermissionByIdError {}
