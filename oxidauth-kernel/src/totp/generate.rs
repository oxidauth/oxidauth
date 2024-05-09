use crate::dev_prelude::*;
use std::{
    error::Error,
    fmt::{self},
};

pub use super::TOTPCode;

pub type GenerateTOTPService = Arc<
    dyn for<'a> Service<
        &'a GenerateTOTP,
        Response = TOTPCode,
        Error = BoxedError,
    >,
>;

#[derive(Debug)]
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
