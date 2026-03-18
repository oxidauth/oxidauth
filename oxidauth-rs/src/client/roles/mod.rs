pub mod permissions;
#[allow(clippy::module_inception)]
pub mod roles;

pub mod create_role;
pub mod delete_role;
pub mod find_role_by_id;
pub mod find_role_by_name;
pub mod list_all_roles;
pub mod update_role;

pub use oxidauth_kernel::roles::Role;

#[cfg(feature = "mock")]
use super::mock::ClientMock;
use super::{
    Client,
    Resource,
    fmt,
    handle_response,
};
use crate::roles::{
    create_role::CreateRoleTrait,
    delete_role::DeleteRoleTrait,
    find_role_by_id::FindRoleByIdTrait,
    find_role_by_name::FindRoleByNameTrait,
    list_all_roles::ListAllRolesTrait,
    update_role::UpdateRoleTrait,
};

pub trait RolesTrait:
    UpdateRoleTrait
    + ListAllRolesTrait
    + FindRoleByNameTrait
    + FindRoleByIdTrait
    + DeleteRoleTrait
    + CreateRoleTrait
{
}

impl RolesTrait for Client {
}

#[cfg(feature = "mock")]
impl RolesTrait for ClientMock {
}
