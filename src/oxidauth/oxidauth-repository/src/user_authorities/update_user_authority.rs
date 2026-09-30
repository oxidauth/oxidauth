use std::error::Error;

pub use oxidauth_kernel::{
    error::BoxedError,
    user_authorities::{UserAuthority, update_user_authority::UpdateUserAuthority},
};

pub use crate::prelude::*;

#[async_trait]
pub trait UpdateUserAuthorityQuery: Send + Sync + 'static {
    async fn update_user_authority(
        &self,
        params: &UpdateUserAuthority,
    ) -> Result<UserAuthority, BoxedError>;
}
#[derive(Debug)]
pub struct UpdateUserAuthorityError {
    pub reason: String,
    pub source: Box<dyn Error>,
}
