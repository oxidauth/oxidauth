use oxidauth_kernel::user_role_grants::list_user_role_grants_by_user_id::{
    ListUserRoleGrantsByUserId,
    UserRole,
};
use serde::{Deserialize, Serialize};

pub type ListUserRoleGrantsByUserIdReq = ListUserRoleGrantsByUserId;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListUserRoleGrantsByUserIdRes {
    pub user_role_grants: Vec<UserRole>,
}
