use crate::dev_prelude::*;

pub use super::UserPermission;

#[async_trait]
pub trait ListUserPermissionGrantsByUserIdTrait: Send + Sync + 'static {
    async fn list_user_permission_grants_by_user_id(
        &self,
        params: &ListUserPermissionGrantsByUserId,
    ) -> Result<Vec<UserPermission>, BoxedError>;
}

pub type ListUserPermissionGrantsByUserIdService = Arc<dyn ListUserPermissionGrantsByUserIdTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListUserPermissionGrantsByUserId {
    pub user_id: Uuid,
}
