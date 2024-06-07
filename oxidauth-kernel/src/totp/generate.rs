use crate::dev_prelude::*;
use std::{
    error::Error,
    fmt::{self},
};

pub type GenerateTOTPService = Arc<
    dyn for<'a> Service<
        &'a GenerateTOTP,
        Response = TOTPCode,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct TOTPCode {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateTOTP {
    pub user_id: Uuid,
}

#[derive(Debug)]
pub struct GenerateTOTPError;

impl fmt::Display for GenerateTOTPError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "could not generate TOTP")
    }
}

impl Error for GenerateTOTPError {}
