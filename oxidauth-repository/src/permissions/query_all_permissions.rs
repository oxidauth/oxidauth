use crate::prelude::*;

pub use super::PermissionRow;

#[async_trait]
pub trait QueryAllPermissions: Send + Sync + 'static {
    async fn query_all_permissions(
        &self,
    ) -> Result<Vec<PermissionRow>, QueryAllPermissionsError>;
}

#[derive(Debug)]
pub struct QueryAllPermissionsError {}
