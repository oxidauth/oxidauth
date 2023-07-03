use crate::dev_prelude::*;

use super::*;

#[async_trait]
pub trait UserAuthorityCreateService: Send + Sync + 'static {
    async fn create_user_authority(
        &self,
        params: &UserAuthorityCreate,
    ) -> Result<UserAuthority, UserAuthorityCreateError>;
}

#[derive(Debug)]
pub struct UserAuthorityCreate {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub user_identifier: String,
    pub params: Value,
}

#[derive(Debug)]
pub struct UserAuthorityCreateError {}
