pub use super::UserPermission;
use crate::dev_prelude::*;

#[async_trait]
pub trait DeleteUserPermissionGrantServiceTrait: Send + Sync + 'static {
    async fn delete_user_permission_grant(
        &self,
        params: &DeleteUserPermission,
    ) -> Result<UserPermission, BoxedError>;
}

pub type DeleteUserPermissionGrantService = Arc<dyn DeleteUserPermissionGrantServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteUserPermission {
    pub user_id: Uuid,
    pub permission: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteUserPermissionGrant {
    pub user_id: Uuid,
    pub permission_id: Uuid,
}
