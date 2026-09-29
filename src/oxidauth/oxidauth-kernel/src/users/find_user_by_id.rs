use crate::dev_prelude::*;

pub use super::User;

pub type FindUserByIdService = Arc<
    dyn for<'a> Service<&'a FindUserById, Response = User, Error = BoxedError>,
>;

#[derive(Debug, Deserialize)]
pub struct FindUserById {
    pub user_id: Uuid,
}
