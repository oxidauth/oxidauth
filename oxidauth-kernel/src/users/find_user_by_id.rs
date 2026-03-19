use crate::dev_prelude::*;

pub use super::User;

#[async_trait]
pub trait FindUserByIdTrait: Send + Sync + 'static {
    async fn find_user_by_id(
        &self,
        params: &FindUserById,
    ) -> Result<User, BoxedError>;
}

pub type FindUserByIdService = Arc<dyn FindUserByIdTrait>;

#[derive(Debug, Deserialize)]
pub struct FindUserById {
    pub user_id: Uuid,
}
