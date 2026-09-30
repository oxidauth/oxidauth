use oxidauth_kernel::users::create_user::{CreateUser, User};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserReq {
    pub user: CreateUser,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRes {
    pub user: User,
}
