use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::TOTPSecret;
pub use crate::error::BoxedError;

#[async_trait]
pub trait FindTOTPSecretByUserIdServiceTrait: Send + Sync + 'static {
    async fn find_totp_secret_by_user_id(
        &self,
        params: &FindTOTPSecretByUserId,
    ) -> Result<TOTPSecret, BoxedError>;
}

pub type FindTOTPSecretByUserIdService = Arc<dyn FindTOTPSecretByUserIdServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindTOTPSecretByUserId {
    pub user_id: Uuid,
}
