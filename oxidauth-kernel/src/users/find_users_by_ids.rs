use crate::dev_prelude::*;

pub use super::User;

pub type FindUsersByIdsService = Arc<
    dyn for<'a> Service<
        &'a FindUsersByIds,
        Response = (Vec<User>, Vec<Uuid>),
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindUsersByIds {
    pub user_ids: Vec<Uuid>,
}
