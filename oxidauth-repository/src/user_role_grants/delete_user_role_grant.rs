use crate::prelude::*;

use super::UserRoleGrantRow;

#[async_trait]
pub trait DeleteUserRoleGrant: Send + Sync + 'static {
    async fn delete_user_role_grant(
        &self,
        params: &DeleteUserRoleGrantParams,
    ) -> Result<UserRoleGrantRow, DeleteUserRoleGrantError>;
}

#[derive(Debug)]
pub struct DeleteUserRoleGrantParams {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[derive(Debug)]
pub struct DeleteUserRoleGrantError {}
