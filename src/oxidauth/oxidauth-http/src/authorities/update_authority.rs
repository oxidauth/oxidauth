use oxidauth_kernel::authorities::update_authority::{Authority, UpdateAuthority};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAuthorityPathReq {
    pub authority_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAuthorityReq {
    pub authority: UpdateAuthority,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAuthorityRes {
    pub authority: Authority,
}
