pub use super::UserRole;
use crate::dev_prelude::*;

#[async_trait]
pub trait ListUserRoleGrantsByUserIdServiceTrait: Send + Sync + 'static {
    async fn list_user_role_grants_by_user_id(
        &self,
        params: &ListUserRoleGrantsByUserId,
    ) -> Result<Vec<UserRole>, BoxedError>;
}

pub type ListUserRoleGrantsByUserIdService = Arc<dyn ListUserRoleGrantsByUserIdServiceTrait>;

#[derive(Debug, Deserialize)]
pub struct ListUserRoleGrantsByUserId {
    pub user_id: Uuid,
}
