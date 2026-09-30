use std::error::Error;

use oxidauth_kernel::user_role_grants::{
    UserRoleGrant,
    create_user_role_grant::CreateUserRoleGrant,
};
pub use oxidauth_kernel::users::User;

pub use crate::prelude::*;

#[async_trait]
pub trait InsertUserRoleGrantQuery: Send + Sync + 'static {
    async fn insert_user_role_grant(
        &self,
        params: &CreateUserRoleGrant,
    ) -> Result<UserRoleGrant, BoxedError>;
}
#[derive(Debug)]
pub struct InsertUserRoleGrantError {
    pub reason: String,
    pub source: Box<dyn Error>,
}
