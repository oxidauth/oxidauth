use crate::prelude::*;

use super::RoleRoleGrantRow;

#[async_trait]
pub trait DeleteRoleRoleGrant: Send + Sync + 'static {
    async fn delete_role_role_grant(
        &self,
        params: DeleteRoleRoleGrantParams,
    ) -> Result<RoleRoleGrantRow, DeleteRoleRoleGrantError>;
}

#[derive(Debug)]
pub struct DeleteRoleRoleGrantParams {
    pub parent_id: Uuid,
    pub child_id: Uuid,
}

#[derive(Debug)]
pub struct DeleteRoleRoleGrantError {}
