use crate::dev_prelude::*;

pub use super::UserAuthorityWithAuthority;

pub type FindUserAuthorityByUserIdAndAuthorityIdService = Arc<
    dyn for<'a> Service<
        &'a FindUserAuthorityByUserIdAndAuthorityId,
        Response = UserAuthorityWithAuthority,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindUserAuthorityByUserIdAndAuthorityId {
    pub user_id: Uuid,
    pub authority_id: Uuid,
}
