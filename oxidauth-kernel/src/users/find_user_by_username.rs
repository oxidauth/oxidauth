use crate::dev_prelude::*;

pub use super::User;
use super::Username;

pub type FindUserByUsernameService =
    Arc<dyn Service<FindUserByUsername, Response = User, Error = BoxedError>>;

#[derive(Debug)]
pub struct FindUserByUsername {
    pub username: Username,
}
