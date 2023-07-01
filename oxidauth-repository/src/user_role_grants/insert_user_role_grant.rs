use crate::prelude::*;

pub use super::UserRoleGrantRow;

#[async_trait]
pub trait InsertUserRoleGrant: Send + Sync + 'static {
    async fn insert_user_role_grant(
        &self,
        params: &InsertUserRoleGrantParams,
    ) -> Result<UserRoleGrantRow, InsertUserRoleGrantError>;
}

#[derive(Debug)]
pub struct InsertUserRoleGrantParams {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[derive(Debug)]
pub struct InsertUserRoleGrantError {}
