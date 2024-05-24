use crate::dev_prelude::*;
use std::{
    error::Error,
    fmt::{self},
};

pub use super::TOTPGenerationRes;

pub type GenerateTOTPService = Arc<
    dyn for<'a> Service<
        &'a GenerateTOTP,
        Response = TOTPGenerationRes,
        Error = BoxedError,
    >,
>;

pub trait GenerateTOTPTrait:
    for<'a> Service<
    &'a GenerateTOTP,
    Response = TOTPGenerationRes,
    Error = BoxedError,
>
{
}

impl<T> GenerateTOTPTrait for T where
    T: for<'a> Service<
        &'a GenerateTOTP,
        Response = TOTPGenerationRes,
        Error = BoxedError,
    >
{
}

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
