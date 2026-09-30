pub use super::UserAuthorityWithAuthority;
use crate::dev_prelude::*;

#[async_trait]
pub trait FindUserAuthorityByUserIdAndAuthorityIdServiceTrait: Send + Sync + 'static {
    async fn find_user_authority_by_user_id_and_authority_id(
        &self,
        params: &FindUserAuthorityByUserIdAndAuthorityId,
    ) -> Result<UserAuthorityWithAuthority, BoxedError>;
}

pub type FindUserAuthorityByUserIdAndAuthorityIdService =
    Arc<dyn FindUserAuthorityByUserIdAndAuthorityIdServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindUserAuthorityByUserIdAndAuthorityId {
    pub user_id: Uuid,
    pub authority_id: Uuid,
}
