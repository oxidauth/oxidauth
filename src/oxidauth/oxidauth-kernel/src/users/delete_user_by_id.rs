use crate::dev_prelude::*;

pub use super::User;

pub type DeleteUserByIdService = Arc<
    dyn for<'a> Service<
        &'a DeleteUserById,
        Response = User,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct DeleteUserById {
    pub user_id: Uuid,
}
