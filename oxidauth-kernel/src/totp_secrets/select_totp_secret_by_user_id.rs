use std::sync::Arc;
use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::error::BoxedError;
pub use crate::service::Service;

use super::TOTPSecret;

pub type SelectTOTPSecretByUserIdService = Arc<
    dyn for<'a> Service<
        &'a SelectTOTPSecretByUserId,
        Response = TOTPSecret,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct SelectTOTPSecretByUserId {
    pub user_id: Uuid,
}

#[derive(Debug)]
pub struct SelectTOTPSecretByUserIdError;

impl fmt::Display for SelectTOTPSecretByUserIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "could not find TOTP secret for user id"
        )
    }
}

impl Error for SelectTOTPSecretByUserIdError {}
