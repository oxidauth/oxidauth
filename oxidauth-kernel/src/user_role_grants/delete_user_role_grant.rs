use crate::dev_prelude::*;

pub use super::UserRole;

pub type DeleteUserRoleGrantService = Arc<
    dyn for<'a> Service<
        &'a DeleteUserRoleGrant,
        Response = UserRole,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct DeleteUserRoleGrant {
    pub user_id: Uuid,
    pub role_id: Uuid,
}
