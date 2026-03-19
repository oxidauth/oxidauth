use crate::dev_prelude::*;

pub use super::UserPermission;

#[async_trait]
pub trait DeleteUserPermissionGrantTrait: Send + Sync + 'static {
    async fn delete_user_permission_grant(
        &self,
        params: &DeleteUserPermission,
    ) -> Result<UserPermission, BoxedError>;
}

pub type DeleteUserPermissionGrantService = Arc<dyn DeleteUserPermissionGrantTrait>;

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
