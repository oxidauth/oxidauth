use oxidauth_kernel::users::find_user_by_id::{FindUserById, User};
use serde::{Deserialize, Serialize};

pub type FindUserByIdReq = FindUserById;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindUserByIdRes {
    pub user: User,
}
