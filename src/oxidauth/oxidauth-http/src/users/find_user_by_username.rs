use oxidauth_kernel::users::find_user_by_username::{FindUserByUsername, User};
use serde::{Deserialize, Serialize};

pub type FindUserByUsernameReq = FindUserByUsername;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindUserByUsernameRes {
    pub user: User,
}
