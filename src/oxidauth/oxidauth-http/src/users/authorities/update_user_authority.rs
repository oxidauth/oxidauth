use oxidauth_kernel::user_authorities::update_user_authority::UserAuthority;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct UpdateUserAuthorityPathReq {
    pub user_id: Uuid,
    pub authority_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserAuthorityBodyReq {
    pub params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserAuthorityRes {
    pub user_authority: UserAuthority,
}
