use crate::prelude::*;

pub use super::PermissionRow;

#[async_trait]
pub trait UpdatePermission: Send + Sync + 'static {
    async fn update_permission(
        &self,
        params: &UpdatePermissionParams,
    ) -> Result<PermissionRow, UpdatePermissionError>;
}

#[derive(Debug)]
pub struct UpdatePermissionParams {
    pub id: Uuid,
    pub realm: String,
    pub resource: String,
    pub action: String,
}

#[derive(Debug)]
pub struct UpdatePermissionError {}
