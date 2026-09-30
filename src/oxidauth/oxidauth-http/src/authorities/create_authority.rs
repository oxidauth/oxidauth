use oxidauth_kernel::authorities::create_authority::{Authority, CreateAuthority};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAuthorityReq {
    pub authority: CreateAuthority,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAuthorityRes {
    pub authority: Authority,
}
