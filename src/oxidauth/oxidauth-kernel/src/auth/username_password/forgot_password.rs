use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::BoxedError;

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgotPasswordParams {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgotPasswordResponse {
    pub code: String,
}

#[async_trait]
pub trait ForgotPasswordServiceTrait: Send + Sync + 'static {
    async fn forgot_password(
        &self,
        params: &ForgotPasswordParams,
    ) -> Result<ForgotPasswordResponse, BoxedError>;
}

pub type ForgotPasswordService = Arc<dyn ForgotPasswordServiceTrait>;
