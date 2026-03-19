use crate::dev_prelude::*;

pub use super::UserRole;

#[async_trait]
pub trait DeleteUserRoleGrantTrait: Send + Sync + 'static {
    async fn delete_user_role_grant(
        &self,
        params: &DeleteUserRoleGrant,
    ) -> Result<UserRole, BoxedError>;
}

pub type DeleteUserRoleGrantService = Arc<dyn DeleteUserRoleGrantTrait>;

#[derive(Debug, Deserialize)]
pub struct DeleteUserRoleGrant {
    pub user_id: Uuid,
    pub role_id: Uuid,
}
