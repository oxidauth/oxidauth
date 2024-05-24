use crate::dev_prelude::*;

use std::{
    error::Error,
    fmt::{self},
};

use super::TOTPValidationRes;

pub type ValidateTOTPService = Arc<
    dyn for<'a> Service<
        &'a ValidateTOTP,
        Response = TOTPValidationRes,
        Error = BoxedError,
    >,
>;

#[derive(Debug)]
pub struct ValidateTOTP {
    pub user_id: Uuid,
    pub code: String,
    pub client_key: Uuid,
}

#[derive(Debug)]
pub struct ValidateTOTPError;

impl fmt::Display for ValidateTOTPError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "could not validate TOTP")
    }
}

impl Error for ValidateTOTPError {}
