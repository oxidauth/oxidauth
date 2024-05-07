use crate::dev_prelude::*;

pub use super::TOTPCode;

pub type GenerateTOTPService = Arc<
    dyn for<'a> Service<
        &'a GenerateTOTP,
        Response = TOTPCode,
        Error = BoxedError,
    >,
>;

#[derive(Debug)]
pub struct GenerateTOTP;
