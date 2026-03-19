use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::BoxedError;

pub use super::UserAuthorityWithAuthority;

#[async_trait]
pub trait ListUserAuthoritiesByUserIdTrait: Send + Sync + 'static {
    async fn list_user_authorities_by_user_id(
        &self,
        params: &ListUserAuthoritiesByUserId,
    ) -> Result<Vec<UserAuthorityWithAuthority>, BoxedError>;
}

pub type ListUserAuthoritiesByUserIdService = Arc<dyn ListUserAuthoritiesByUserIdTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListUserAuthoritiesByUserId {
    pub user_id: Uuid,
}
