use oxidauth_kernel::auth::authenticate::AuthenticateParams;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type AuthenticateReq = AuthenticateParams;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticateRes {
    pub jwt: String,
    pub refresh_token: Uuid,
}
