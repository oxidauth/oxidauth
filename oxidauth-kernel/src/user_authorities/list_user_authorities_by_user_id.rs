use crate::dev_prelude::*;

pub use super::UserAuthorityWithAuthority;

pub type ListUserAuthoritiesByUserIdService = Arc<
    dyn for<'a> Service<
        &'a ListUserAuthoritiesByUserId,
        Response = Vec<UserAuthorityWithAuthority>,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct ListUserAuthoritiesByUserId {
    pub user_id: Uuid,
}
