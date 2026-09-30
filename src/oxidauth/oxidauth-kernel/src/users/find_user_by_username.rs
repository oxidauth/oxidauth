pub use super::User;
use super::Username;
use crate::dev_prelude::*;

#[async_trait]
pub trait FindUserByUsernameServiceTrait: Send + Sync + 'static {
    async fn find_user_by_username(&self, params: &FindUserByUsername) -> Result<User, BoxedError>;
}

pub type FindUserByUsernameService = Arc<dyn FindUserByUsernameServiceTrait>;

#[derive(Debug, Deserialize)]
pub struct FindUserByUsername {
    pub username: Username,
}
