use std::error::Error;

use oxidauth_kernel::user_permission_grants::{
    UserPermissionGrant,
    delete_user_permission_grant::DeleteUserPermissionGrant,
};
pub use oxidauth_kernel::users::User;

pub use crate::prelude::*;

#[async_trait]
pub trait DeleteUserPermissionGrantQuery: Send + Sync + 'static {
    async fn delete_user_permission_grant(
        &self,
        params: &DeleteUserPermissionGrant,
    ) -> Result<UserPermissionGrant, BoxedError>;
}
#[derive(Debug)]
pub struct DeleteUserPermissionGrantError {
    pub reason: String,
    pub source: Box<dyn Error>,
}
