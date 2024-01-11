use crate::dev_prelude::*;

pub use super::User;
use super::Username;

pub type FindUserByUsernameService = Arc<
    dyn for<'a> Service<
        &'a FindUserByUsername,
        Response = User,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct FindUserByUsername {
    pub username: Username,
}
