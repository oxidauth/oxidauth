use crate::dev_prelude::*;

pub use super::User;

#[async_trait]
pub trait DeleteUserByIdTrait: Send + Sync + 'static {
    async fn delete_user_by_id(
        &self,
        params: &DeleteUserById,
    ) -> Result<User, BoxedError>;
}

pub type DeleteUserByIdService = Arc<dyn DeleteUserByIdTrait>;

#[derive(Debug, Deserialize)]
pub struct DeleteUserById {
    pub user_id: Uuid,
}
