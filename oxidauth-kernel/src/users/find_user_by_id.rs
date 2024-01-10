use crate::dev_prelude::*;

pub use super::User;

pub type FindUserByIdService =
    Arc<dyn for<'a> Service<Uuid, Response = User, Error = BoxedError>>;
