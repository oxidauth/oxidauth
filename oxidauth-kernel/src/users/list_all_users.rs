use crate::dev_prelude::*;

pub use super::User;

pub type ListAllUsersService = Arc<
    dyn for<'a> Service<
        &'a ListAllUsers,
        Response = Vec<User>,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct ListAllUsers {}
