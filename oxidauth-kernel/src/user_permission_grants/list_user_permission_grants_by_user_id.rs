use crate::dev_prelude::*;

pub use super::UserPermission;

pub type ListUserPermissionGrantsByUserIdService = Arc<
    dyn for<'a> Service<
        &'a ListUserPermissionGrantsByUserId,
        Response = Vec<UserPermission>,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct ListUserPermissionGrantsByUserId {
    pub user_id: Uuid,
}
