use crate::dev_prelude::*;

pub use super::UserRole;

#[async_trait]
pub trait ListUserRoleGrantsByUserIdTrait: Send + Sync + 'static {
    async fn list_user_role_grants_by_user_id(
        &self,
        params: &ListUserRoleGrantsByUserId,
    ) -> Result<Vec<UserRole>, BoxedError>;
}

pub type ListUserRoleGrantsByUserIdService = Arc<dyn ListUserRoleGrantsByUserIdTrait>;

#[derive(Debug, Deserialize)]
pub struct ListUserRoleGrantsByUserId {
    pub user_id: Uuid,
}
