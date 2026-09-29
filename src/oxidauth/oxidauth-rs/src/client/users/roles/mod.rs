pub mod create_user_role;
pub mod delete_user_role;
pub mod list_user_roles_by_user_id;

use super::*;
#[cfg(feature = "mock")]
use crate::mock::ClientMock;
pub use crate::users::roles::{
    create_user_role::CreateUserRoleTrait,
    delete_user_role::DeleteUserRoleTrait,
    list_user_roles_by_user_id::ListUserRolesByUserIdTrait,
};

pub trait UserRolesTrait:
    ListUserRolesByUserIdTrait + DeleteUserRoleTrait + CreateUserRoleTrait
{
}

impl UserRolesTrait for Client {
}

#[cfg(feature = "mock")]
impl UserRolesTrait for ClientMock {
}
