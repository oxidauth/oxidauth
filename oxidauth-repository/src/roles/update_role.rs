use crate::prelude::*;

use super::RoleRow;

#[async_trait]
pub trait UpdateRole: Send + Sync + 'static {
    async fn update_role(&self, update_role: &UpdateRoleParams)
        -> Result<RoleRow, UpdateRoleError>;
}

#[derive(Debug)]
pub struct UpdateRoleParams {
    pub name: String,
}

#[derive(Debug)]
pub struct UpdateRoleError {}
