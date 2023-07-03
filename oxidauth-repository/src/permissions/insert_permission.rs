use crate::prelude::*;

pub use super::PermissionRow;

#[async_trait]
pub trait InsertPermission: Send + Sync + 'static {
    async fn insert_permission(
        &self,
        params: &InsertPermissionParams,
    ) -> Result<PermissionRow, InsertPermissionError>;
}

#[derive(Debug)]
pub struct InsertPermissionParams {
    pub id: Option<Uuid>,
    pub realm: String,
    pub resource: String,
    pub action: String,
}

#[derive(Debug)]
pub struct InsertPermissionError {}
