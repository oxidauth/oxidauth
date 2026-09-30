pub use oxidauth_kernel::permissions::Permission;
use oxidauth_kernel::permissions::create_permission::CreatePermission;

pub use crate::prelude::*;

#[async_trait]
pub trait InsertPermissionQuery: Send + Sync + 'static {
    async fn insert_permission(&self, params: &CreatePermission) -> Result<Permission, BoxedError>;
}
