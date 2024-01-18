use crate::dev_prelude::*;

pub use super::UserAuthority;

pub type DeleteUserAuthorityService = Arc<
    dyn for<'a> Service<
        &'a DeleteUserAuthority,
        Response = UserAuthority,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct DeleteUserAuthority {
    pub user_id: Uuid,
    pub authority_id: Uuid,
}
