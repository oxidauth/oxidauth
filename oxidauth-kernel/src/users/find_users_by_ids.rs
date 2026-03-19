use crate::dev_prelude::*;

pub use super::User;

#[async_trait]
pub trait FindUsersByIdsTrait: Send + Sync + 'static {
    async fn find_users_by_ids(
        &self,
        params: &FindUsersByIds,
    ) -> Result<UsersByIds, BoxedError>;
}

pub type FindUsersByIdsService = Arc<dyn FindUsersByIdsTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindUsersByIds {
    pub user_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsersByIds {
    pub users: Vec<User>,
    pub user_ids_not_found: Vec<Uuid>,
}
