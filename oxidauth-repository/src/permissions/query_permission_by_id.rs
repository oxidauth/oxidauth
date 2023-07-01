use crate::prelude::*;

pub use super::PermissionRow;

#[async_trait]
pub trait QueryPermissionById: Send + Sync + 'static {
    async fn query_permission_by_id(
        &self,
        permission_id: Uuid,
    ) -> Result<PermissionRow, QueryPermissionByIdError>;
}

#[derive(Debug)]
pub struct QueryPermissionByIdError {}
