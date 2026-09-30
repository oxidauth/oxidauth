use std::error::Error;

use oxidauth_kernel::user_authorities::{
    UserAuthority,
    delete_user_authority::DeleteUserAuthority,
};
pub use oxidauth_kernel::users::User;

pub use crate::prelude::*;

#[async_trait]
pub trait DeleteUserAuthorityQuery: Send + Sync + 'static {
    async fn delete_user_authority(
        &self,
        params: &DeleteUserAuthority,
    ) -> Result<UserAuthority, BoxedError>;
}
#[derive(Debug)]
pub struct DeleteUserAuthorityError {
    pub reason: String,
    pub source: Box<dyn Error>,
}
