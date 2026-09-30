pub use oxidauth_kernel::auth::tree::*;

pub use crate::prelude::*;

#[async_trait]
pub trait PermissionTreeQuery: Send + Sync + 'static {
    async fn permission_tree(
        &self,
        params: &PermissionSearch,
    ) -> Result<PermissionsResponse, BoxedError>;
}
