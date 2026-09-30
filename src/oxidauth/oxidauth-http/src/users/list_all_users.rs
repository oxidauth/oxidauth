use oxidauth_kernel::users::list_all_users::{ListAllUsers, User};
use serde::{Deserialize, Serialize};

pub type ListAllUsersReq = ListAllUsers;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAllUsersRes {
    pub users: Vec<User>,
}
