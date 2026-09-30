pub use super::User;
use crate::dev_prelude::*;

#[async_trait]
pub trait FindUserByIdServiceTrait: Send + Sync + 'static {
    async fn find_user_by_id(&self, params: &FindUserById) -> Result<User, BoxedError>;
}

pub type FindUserByIdService = Arc<dyn FindUserByIdServiceTrait>;

#[derive(Debug, Deserialize)]
pub struct FindUserById {
    pub user_id: Uuid,
}
