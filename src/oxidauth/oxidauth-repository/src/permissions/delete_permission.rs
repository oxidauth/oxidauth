pub use oxidauth_kernel::permissions::Permission;
use oxidauth_kernel::permissions::delete_permission::DeletePermission;

pub use crate::prelude::*;

#[async_trait]
pub trait DeletePermissionQuery: Send + Sync + 'static {
    async fn delete_permission(&self, params: &DeletePermission) -> Result<Permission, BoxedError>;
}
