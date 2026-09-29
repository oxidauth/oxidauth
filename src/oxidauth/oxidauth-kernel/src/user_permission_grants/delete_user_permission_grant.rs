use crate::dev_prelude::*;

pub use super::UserPermission;

pub type DeleteUserPermissionGrantService = Arc<
    dyn for<'a> Service<
        &'a DeleteUserPermission,
        Response = UserPermission,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteUserPermission {
    pub user_id: Uuid,
    pub permission: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteUserPermissionGrant {
    pub user_id: Uuid,
    pub permission_id: Uuid,
}
