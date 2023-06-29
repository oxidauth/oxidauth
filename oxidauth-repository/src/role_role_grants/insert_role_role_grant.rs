use crate::prelude::*;

use super::RoleRoleGrantRow;

#[async_trait]
pub trait InsertRoleRoleGrant: Send + Sync + 'static {
    async fn insert_role_role_grant(
        &self,
        params: &InsertRoleRoleGrantParams,
    ) -> Result<RoleRoleGrantRow, InsertRoleRoleGrantError>;
}

#[derive(Debug)]
pub struct InsertRoleRoleGrantParams {
    pub parent_id: Uuid,
    pub child_id: Uuid,
}

#[derive(Debug)]
pub struct InsertRoleRoleGrantError {}
