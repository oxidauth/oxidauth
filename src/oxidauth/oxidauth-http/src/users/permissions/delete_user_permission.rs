use oxidauth_kernel::user_permission_grants::delete_user_permission_grant::{
    DeleteUserPermission,
    UserPermission,
};
use serde::{Deserialize, Serialize};

pub type DeleteUserPermissionReq = DeleteUserPermission;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteUserPermissionRes {
    pub user_permission: UserPermission,
}
