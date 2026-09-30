pub use oxidauth_kernel::permissions::Permission;
use oxidauth_kernel::permissions::find_permission_by_parts::*;

pub use crate::prelude::*;

#[async_trait]
pub trait SelectPermissionByPartsQuery: Send + Sync + 'static {
    async fn select_permission_by_parts(
        &self,
        params: &FindPermissionByParts,
    ) -> Result<Option<Permission>, BoxedError>;
}
