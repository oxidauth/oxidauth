use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::error::BoxedError;
pub use crate::service::Service;

use super::TOTPSecret;

pub type FindTOTPSecretByUserIdService = Arc<
    dyn for<'a> Service<
        &'a FindTOTPSecretByUserId,
        Response = TOTPSecret,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindTOTPSecretByUserId {
    pub user_id: Uuid,
}
