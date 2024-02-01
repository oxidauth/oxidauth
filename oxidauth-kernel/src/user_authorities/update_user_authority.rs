use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt, sync::Arc};
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::UserAuthority;

pub type UpdateUserAuthorityService = Arc<
    dyn for<'a> Service<
        &'a UpdateUserAuthority,
        Response = UserAuthority,
        Error = BoxedError,
    >,
>;

#[derive(Serialize, Deserialize)]
pub struct UpdateUserAuthority {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub params: Value,
}

impl fmt::Debug for UpdateUserAuthority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UpdateUserAuthority")
            .field("user_id", &self.user_id)
            .field(
                "authority_id",
                &self.authority_id,
            )
            .finish()
    }
}
