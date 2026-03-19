use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::BoxedError;

pub use super::UserAuthorityWithAuthority;

#[async_trait]
pub trait FindUserAuthorityByUserIdAndAuthorityIdTrait: Send + Sync + 'static {
    async fn find_user_authority_by_user_id_and_authority_id(
        &self,
        params: &FindUserAuthorityByUserIdAndAuthorityId,
    ) -> Result<UserAuthorityWithAuthority, BoxedError>;
}

pub type FindUserAuthorityByUserIdAndAuthorityIdService = Arc<dyn FindUserAuthorityByUserIdAndAuthorityIdTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindUserAuthorityByUserIdAndAuthorityId {
    pub user_id: Uuid,
    pub authority_id: Uuid,
}
