pub use super::UserAuthority;
use crate::dev_prelude::*;

#[async_trait]
pub trait DeleteUserAuthorityServiceTrait: Send + Sync + 'static {
    async fn delete_user_authority(
        &self,
        params: &DeleteUserAuthority,
    ) -> Result<UserAuthority, BoxedError>;
}

pub type DeleteUserAuthorityService = Arc<dyn DeleteUserAuthorityServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteUserAuthority {
    pub user_id: Uuid,
    pub authority_id: Uuid,
}
