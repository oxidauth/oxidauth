use oxidauth_kernel::user_role_grants::UserRoleGrant;

use crate::prelude::*;

pub use super::UserRoleGrantRow;

#[async_trait]
pub trait DeleteUserRoleGrant: Send + Sync + 'static {
    async fn delete_user_role_grant(
        &self,
        params: &DeleteUserRoleGrantParams,
    ) -> Result<UserRoleGrant, DeleteUserRoleGrantError>;
}

#[derive(Debug)]
pub struct DeleteUserRoleGrantParams {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[derive(Debug)]
pub struct DeleteUserRoleGrantError {}
