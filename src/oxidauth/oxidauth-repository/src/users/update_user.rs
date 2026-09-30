use std::error::Error;

pub use oxidauth_kernel::{
    error::BoxedError,
    users::{User, update_user::UpdateUser},
};

pub use crate::prelude::*;

#[async_trait]
pub trait UpdateUserQuery: Send + Sync + 'static {
    async fn update_user(&self, params: &UpdateUser) -> Result<User, BoxedError>;
}
#[derive(Debug)]
pub struct UpdateUserError {
    pub reason: String,
    pub source: Box<dyn Error>,
}
