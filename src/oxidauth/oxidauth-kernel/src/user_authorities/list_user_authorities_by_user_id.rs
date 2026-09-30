pub use super::UserAuthorityWithAuthority;
use crate::dev_prelude::*;

#[async_trait]
pub trait ListUserAuthoritiesByUserIdServiceTrait: Send + Sync + 'static {
    async fn list_user_authorities_by_user_id(
        &self,
        params: &ListUserAuthoritiesByUserId,
    ) -> Result<Vec<UserAuthorityWithAuthority>, BoxedError>;
}

pub type ListUserAuthoritiesByUserIdService = Arc<dyn ListUserAuthoritiesByUserIdServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListUserAuthoritiesByUserId {
    pub user_id: Uuid,
}
