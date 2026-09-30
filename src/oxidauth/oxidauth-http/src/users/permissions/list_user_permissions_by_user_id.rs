use oxidauth_kernel::user_permission_grants::list_user_permission_grants_by_user_id::{
    ListUserPermissionGrantsByUserId,
    UserPermission,
};
use serde::{Deserialize, Serialize};

pub type ListUserPermissionGrantsByUserIdReq = ListUserPermissionGrantsByUserId;

#[derive(Debug, Deserialize, Serialize)]
pub struct ListUserPermissionGrantsByUserIdRes {
    pub user_permission_grants: Vec<UserPermission>,
}
