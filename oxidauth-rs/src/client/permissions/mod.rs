pub mod create_permission;
pub mod delete_permission;
pub mod find_permission_by_parts;
pub mod list_all_permissions;

#[cfg(feature = "mock")]
use super::mock::ClientMock;
use super::{
    Client,
    Resource,
    fmt,
    handle_response,
};
use crate::client::permissions::{
    create_permission::CreatePermissionTrait,
    delete_permission::DeletePermissionTrait,
    find_permission_by_parts::FindPermissionByPartsTrait,
    list_all_permissions::ListAllPermissionsTrait,
};

pub trait PermissionsTrait:
    ListAllPermissionsTrait
    + FindPermissionByPartsTrait
    + DeletePermissionTrait
    + CreatePermissionTrait
{
}

impl PermissionsTrait for Client {
}

#[cfg(feature = "mock")]
impl PermissionsTrait for ClientMock {
}
