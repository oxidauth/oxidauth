pub mod create_role_role_grant;
pub mod delete_role_role_grant;
pub mod list_role_role_grants_by_child_id;

pub use super::*;

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

pub use crate::roles::roles::{
    create_role_role_grant::CreateRoleRoleGrantTrait,
    delete_role_role_grant::DeleteRoleRoleGrantTrait,
    list_role_role_grants_by_child_id::ListRoleRoleGrantsByChildIdTrait,
};

pub trait RoleRoleGrantsTrait:
    ListRoleRoleGrantsByChildIdTrait
    + DeleteRoleRoleGrantTrait
    + CreateRoleRoleGrantTrait
{
}

impl RoleRoleGrantsTrait for Client {
}

#[cfg(feature = "mock")]
impl RoleRoleGrantsTrait for ClientMock {
}
