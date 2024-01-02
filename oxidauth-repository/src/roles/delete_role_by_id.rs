use crate::prelude::*;

use super::RoleRow;

#[async_trait]
pub trait DeleteRoleById: Send + Sync + 'static {
    async fn delete_role_by_id(
        &self,
        role_id: Uuid,
    ) -> Result<RoleRow, DeleteRoleByIdError>;
}

#[derive(Debug)]
pub struct DeleteRoleByIdError {}
