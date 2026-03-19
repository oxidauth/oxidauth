use crate::dev_prelude::*;

pub use super::User;
use super::Username;

#[async_trait]
pub trait FindUserByUsernameTrait: Send + Sync + 'static {
    async fn find_user_by_username(
        &self,
        params: &FindUserByUsername,
    ) -> Result<User, BoxedError>;
}

pub type FindUserByUsernameService = Arc<dyn FindUserByUsernameTrait>;

#[derive(Debug, Deserialize)]
pub struct FindUserByUsername {
    pub username: Username,
}
