use oxidauth_kernel::{JsonValue, user_authorities::create_user_authority::UserAuthority};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserAuthorityPathReq {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserAuthorityBodyReq {
    pub client_key: Uuid,
    pub user_authority: UserAuthorityParams,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAuthorityParams {
    pub params: JsonValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserAuthorityRes {
    pub user_authority: UserAuthority,
}
