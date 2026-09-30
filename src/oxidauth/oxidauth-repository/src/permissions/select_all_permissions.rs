pub use oxidauth_kernel::permissions::Permission;
use oxidauth_kernel::permissions::list_all_permissions::*;

pub use crate::prelude::*;

#[async_trait]
pub trait SelectAllPermissionsQuery: Send + Sync + 'static {
    async fn select_all_permissions(
        &self,
        params: &ListAllPermissions,
    ) -> Result<Vec<Permission>, BoxedError>;
}
