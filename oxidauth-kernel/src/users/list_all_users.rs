use crate::dev_prelude::*;

pub use super::User;

#[async_trait]
pub trait ListAllUsersTrait: Send + Sync + 'static {
    async fn list_all_users(
        &self,
        params: &ListAllUsers,
    ) -> Result<Vec<User>, BoxedError>;
}

pub type ListAllUsersService = Arc<dyn ListAllUsersTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAllUsers;
