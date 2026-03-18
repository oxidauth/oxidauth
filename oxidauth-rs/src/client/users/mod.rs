pub mod authorities;
pub mod create_user;
pub mod delete_user;
pub mod find_user_by_id;
pub mod find_user_by_username;
pub mod find_users_by_ids;
pub mod list_all_users;
pub mod permissions;
pub mod roles;
pub mod update_user;

pub use oxidauth_kernel::users::*;

#[cfg(feature = "mock")]
use super::mock::ClientMock;
use super::{
    Client,
    Resource,
    fmt,
    handle_response,
};
use crate::users::{
    create_user::CreateUserTrait,
    delete_user::DeleteUserTrait,
    find_user_by_id::FindUserByIdTrait,
    find_user_by_username::FindUserByUsernameTrait,
    list_all_users::ListAllUsersTrait,
};

pub trait UsersTrait:
    ListAllUsersTrait
    + FindUserByIdTrait
    + FindUserByUsernameTrait
    + FindUserByIdTrait
    + DeleteUserTrait
    + CreateUserTrait
{
}

impl UsersTrait for Client {
}

#[cfg(feature = "mock")]
impl UsersTrait for ClientMock {
}
