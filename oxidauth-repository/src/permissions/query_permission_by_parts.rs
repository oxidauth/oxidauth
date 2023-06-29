use crate::prelude::*;

use super::PermissionRow;

#[async_trait]
pub trait QueryPermissionByParts: Send + Sync + 'static {
    async fn query_permission_by_parts(
        &self,
        params: QueryPermissionByPartsParams,
    ) -> Result<PermissionRow, QueryPermissionByPartsError>;
}

#[derive(Debug)]
pub struct QueryPermissionByPartsParams {
    pub realm: String,
    pub resource: String,
    pub action: String,
}

#[derive(Debug)]
pub struct QueryPermissionByPartsError {}
