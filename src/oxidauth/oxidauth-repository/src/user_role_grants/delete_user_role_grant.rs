use std::error::Error;

use oxidauth_kernel::user_role_grants::{
    UserRoleGrant,
    delete_user_role_grant::DeleteUserRoleGrant,
};
pub use oxidauth_kernel::users::User;

pub use crate::prelude::*;

#[async_trait]
pub trait DeleteUserRoleGrantQuery: Send + Sync + 'static {
    async fn delete_user_role_grant(
        &self,
        params: &DeleteUserRoleGrant,
    ) -> Result<UserRoleGrant, BoxedError>;
}
#[derive(Debug)]
pub struct DeleteUserRoleGrantError {
    pub reason: String,
    pub source: Box<dyn Error>,
}
