pub use super::UserRole;
use crate::dev_prelude::*;

#[async_trait]
pub trait DeleteUserRoleGrantServiceTrait: Send + Sync + 'static {
    async fn delete_user_role_grant(
        &self,
        params: &DeleteUserRoleGrant,
    ) -> Result<UserRole, BoxedError>;
}

pub type DeleteUserRoleGrantService = Arc<dyn DeleteUserRoleGrantServiceTrait>;

#[derive(Debug, Deserialize)]
pub struct DeleteUserRoleGrant {
    pub user_id: Uuid,
    pub role_id: Uuid,
}
