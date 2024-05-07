use serde::{Deserialize, Serialize};

pub mod generate;
pub mod validate;

#[derive(Debug)]
pub struct TOTPCode {
    pub code: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TOTPValidation {
    pub code_validation: bool,
}
