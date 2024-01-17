use crate::dev_prelude::*;

pub use super::UserRole;

pub type ListUserRoleGrantsByUserIdService = Arc<
    dyn for<'a> Service<
        &'a ListUserRoleGrantsByUserId,
        Response = Vec<UserRole>,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct ListUserRoleGrantsByUserId {
    pub user_id: Uuid,
}
