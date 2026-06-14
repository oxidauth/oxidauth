use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgotPasswordParams {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgotPasswordResponse {
    pub code: String,
}

pub type ForgotPasswordService = Arc<
    dyn for<'a> Service<
            &'a ForgotPasswordParams,
            Response = ForgotPasswordResponse,
            Error = BoxedError,
        >,
>;
