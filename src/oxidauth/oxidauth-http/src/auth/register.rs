use oxidauth_kernel::auth::register::RegisterParams;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type RegisterReq = RegisterParams;

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRes {
    pub jwt: String,
    pub refresh_token: Uuid,
}
