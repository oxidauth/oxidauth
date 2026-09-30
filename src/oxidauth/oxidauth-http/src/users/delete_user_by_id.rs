use oxidauth_kernel::users::delete_user_by_id::{DeleteUserById, User};
use serde::{Deserialize, Serialize};

pub type DeleteUserByIdReq = DeleteUserById;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteUserByIdRes {
    pub user: User,
}
