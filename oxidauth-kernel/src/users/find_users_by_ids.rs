use crate::dev_prelude::*;

pub use super::User;

pub type FindUsersByIdsService = Arc<
    dyn for<'a> Service<
        &'a FindUsersByIds,
        Response = UsersByIds,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindUsersByIds {
    pub user_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsersByIds {
    pub users: Vec<User>,
    pub user_ids_not_found: Vec<Uuid>,
}
