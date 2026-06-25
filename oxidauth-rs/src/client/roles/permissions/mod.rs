pub mod create_role_permission_grant;
pub mod delete_role_permission_grant;
pub mod list_role_permission_grants_by_role_id;

use super::*;

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

pub use crate::roles::permissions::{
    create_role_permission_grant::CreateRolePermissionGrantTrait,
    delete_role_permission_grant::DeleteRolePermissionGrantTrait,
    list_role_permission_grants_by_role_id::ListRolePermissionGrantsByRoleIdTrait,
};

pub trait RolePermissionsTrait:
    ListRolePermissionGrantsByRoleIdTrait
    + DeleteRolePermissionGrantTrait
    + CreateRolePermissionGrantTrait
{
}

impl RolePermissionsTrait for Client {
}

#[cfg(feature = "mock")]
impl RolePermissionsTrait for ClientMock {
}
