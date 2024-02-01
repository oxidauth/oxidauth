use std::fmt;

use oxidauth_kernel::user_authorities::create_user_authority::CreateUserAuthority;
pub use oxidauth_kernel::{service::Service, user_authorities::UserAuthority};

pub use crate::prelude::*;

#[async_trait]
pub trait InsertUserAuthorityQuery: Send + Sync + 'static {
    async fn call(
        &self,
        params: impl Into<InsertUserAuthority> + Send + fmt::Debug + 'async_trait,
    ) -> Result<UserAuthority, BoxedError>;
}

pub struct InsertUserAuthority {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub user_identifier: String,
    pub params: Value,
}

impl fmt::Debug for InsertUserAuthority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InsertUserAuthority")
            .field("user_id", &self.user_id)
            .field(
                "authority_id",
                &self.authority_id,
            )
            .field(
                "user_identifier",
                &self.user_identifier,
            )
            .finish()
    }
}

impl From<(Uuid, &CreateUserAuthority)> for InsertUserAuthority {
    fn from(val: (Uuid, &CreateUserAuthority)) -> Self {
        Self {
            user_id: val.0,
            authority_id: val.1.authority_id,
            user_identifier: val.1.user_identifier.clone(),
            params: val.1.params.clone(),
        }
    }
}
