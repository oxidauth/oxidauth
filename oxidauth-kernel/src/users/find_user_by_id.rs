use crate::dev_prelude::*;

pub use super::User;

pub type FindUserByIdService =
    Arc<dyn Service<Uuid, Response = User, Error = BoxedError>>;
