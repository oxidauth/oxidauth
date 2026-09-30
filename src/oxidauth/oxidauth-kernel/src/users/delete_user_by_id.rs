pub use super::User;
use crate::dev_prelude::*;

#[async_trait]
pub trait DeleteUserByIdServiceTrait: Send + Sync + 'static {
    async fn delete_user_by_id(&self, params: &DeleteUserById) -> Result<User, BoxedError>;
}

pub type DeleteUserByIdService = Arc<dyn DeleteUserByIdServiceTrait>;

#[derive(Debug, Deserialize)]
pub struct DeleteUserById {
    pub user_id: Uuid,
}
