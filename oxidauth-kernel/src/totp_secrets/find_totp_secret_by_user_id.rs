use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::error::BoxedError;
pub use crate::service::Service;

use super::TOTPSecret;

#[async_trait]
pub trait FindTotpSecretByUserIdTrait: Send + Sync + 'static {
    async fn find_totp_secret_by_user_id(
        &self,
        params: &FindTOTPSecretByUserId,
    ) -> Result<TOTPSecret, BoxedError>;
}

pub type FindTOTPSecretByUserIdService = Arc<dyn FindTotpSecretByUserIdTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindTOTPSecretByUserId {
    pub user_id: Uuid,
}
