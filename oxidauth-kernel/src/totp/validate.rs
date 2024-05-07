use crate::dev_prelude::*;

use super::TOTPValidation;

pub type ValidateTOTPService = Arc<
    dyn for<'a> Service<
        &'a ValidateTOTP,
        Response = TOTPValidation,
        Error = BoxedError,
    >,
>;

#[derive(Debug)]
pub struct ValidateTOTP;
