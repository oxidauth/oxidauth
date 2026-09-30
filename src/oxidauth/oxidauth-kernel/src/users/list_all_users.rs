pub use super::User;
use crate::dev_prelude::*;

#[async_trait]
pub trait ListAllUsersServiceTrait: Send + Sync + 'static {
    async fn list_all_users(&self, params: &ListAllUsers) -> Result<Vec<User>, BoxedError>;
}

pub type ListAllUsersService = Arc<dyn ListAllUsersServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAllUsers;
