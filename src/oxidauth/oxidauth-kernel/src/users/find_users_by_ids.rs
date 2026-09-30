pub use super::User;
use crate::dev_prelude::*;

#[async_trait]
pub trait FindUsersByIdsServiceTrait: Send + Sync + 'static {
    async fn find_users_by_ids(&self, params: &FindUsersByIds) -> Result<UsersByIds, BoxedError>;
}

pub type FindUsersByIdsService = Arc<dyn FindUsersByIdsServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindUsersByIds {
    pub user_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsersByIds {
    pub users: Vec<User>,
    pub user_ids_not_found: Vec<Uuid>,
}
