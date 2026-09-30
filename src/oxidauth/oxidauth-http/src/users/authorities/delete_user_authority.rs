use oxidauth_kernel::user_authorities::delete_user_authority::{
    DeleteUserAuthority,
    UserAuthority,
};
use serde::{Deserialize, Serialize};

pub type DeleteUserAuthorityReq = DeleteUserAuthority;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteUserAuthorityRes {
    pub user_authority: UserAuthority,
}
