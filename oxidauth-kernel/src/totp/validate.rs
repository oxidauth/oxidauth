use crate::dev_prelude::*;

use super::TOTPValidation;

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateTOTPReq {
    pub user_id: Uuid,
    pub code: u32,
}

pub type ValidateTOTPService = Arc<
    dyn for<'a> Service<
        &'a ValidateTOTP,
        Response = TOTPValidation,
        Error = BoxedError,
    >,
>;

#[derive(Debug)]
pub struct ValidateTOTP {
    pub user_id: Uuid,
    pub code: u32,
}
