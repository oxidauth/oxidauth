pub mod create_user_permission_grant;
pub mod delete_user_permission_grant;
pub mod list_user_permission_grants_by_user_id;

pub use super::*;
#[cfg(feature = "mock")]
use crate::mock::ClientMock;
use crate::users::permissions::{
    create_user_permission_grant::CreateUserPermissionGrantTrait,
    delete_user_permission_grant::DeleteUserPermissionGrantTrait,
    list_user_permission_grants_by_user_id::ListUserPermissionGrantsByUserIdTrait,
};

pub trait UserPermissionsTrait:
    ListUserPermissionGrantsByUserIdTrait
    + DeleteUserPermissionGrantTrait
    + CreateUserPermissionGrantTrait
{
}

impl UserPermissionsTrait for Client {
}

#[cfg(feature = "mock")]
impl UserPermissionsTrait for ClientMock {
}
