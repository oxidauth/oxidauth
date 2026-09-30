pub use super::UserPermission;
use crate::dev_prelude::*;

#[async_trait]
pub trait ListUserPermissionGrantsByUserIdServiceTrait: Send + Sync + 'static {
    async fn list_user_permission_grants_by_user_id(
        &self,
        params: &ListUserPermissionGrantsByUserId,
    ) -> Result<Vec<UserPermission>, BoxedError>;
}

pub type ListUserPermissionGrantsByUserIdService =
    Arc<dyn ListUserPermissionGrantsByUserIdServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListUserPermissionGrantsByUserId {
    pub user_id: Uuid,
}
