use oxidauth_kernel::user_authorities::{
    UserAuthorityWithAuthority,
    list_user_authorities_by_user_id::ListUserAuthoritiesByUserId,
};
use serde::{Deserialize, Serialize};

pub type ListUserAuthoritiesByUserIdReq = ListUserAuthoritiesByUserId;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListUserAuthoritiesByUserIdRes {
    pub user_authorities: Vec<UserAuthorityWithAuthority>,
}
