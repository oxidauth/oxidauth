use std::error::Error;

use oxidauth_kernel::user_permission_grants::{
    UserPermissionGrant,
    create_user_permission_grant::CreateUserPermissionGrant,
};
pub use oxidauth_kernel::users::User;

pub use crate::prelude::*;

#[async_trait]
pub trait InsertUserPermissionGrantQuery: Send + Sync + 'static {
    async fn insert_user_permission_grant(
        &self,
        params: &CreateUserPermissionGrant,
    ) -> Result<UserPermissionGrant, BoxedError>;
}
#[derive(Debug)]
pub struct InsertUserPermissionGrantError {
    pub reason: String,
    pub source: Box<dyn Error>,
}
