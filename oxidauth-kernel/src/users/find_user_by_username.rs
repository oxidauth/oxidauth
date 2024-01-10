use crate::dev_prelude::*;

pub use super::User;

pub type FindUserByUsernameService =
    Arc<dyn Service<String, Response = User, Error = BoxedError>>;
