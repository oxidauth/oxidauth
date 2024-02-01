use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt, sync::Arc};
use uuid::Uuid;

pub use crate::service::Service;
use crate::{
    authorities::{
        find_authority_by_strategy::FindAuthorityByStrategy, AuthorityStrategy,
    },
    error::BoxedError,
};

pub use super::UserAuthority;

pub type CreateUserAuthorityService = Arc<
    dyn for<'a> Service<
        &'a CreateUserAuthorityParams,
        Response = UserAuthority,
        Error = BoxedError,
    >,
>;

#[derive(Serialize, Deserialize)]
pub struct CreateUserAuthorityParams {
    pub user_id: Uuid,
    pub strategy: AuthorityStrategy,
    pub params: Value,
}

impl fmt::Debug for CreateUserAuthorityParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CreateUserAuthorityParams")
            .field("user_id", &self.user_id)
            .field("strategy", &self.strategy)
            .finish()
    }
}

impl From<&CreateUserAuthorityParams> for FindAuthorityByStrategy {
    fn from(value: &CreateUserAuthorityParams) -> Self {
        Self {
            strategy: value.strategy,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateUserAuthority {
    pub authority_id: Uuid,
    pub user_identifier: String,
    pub params: Value,
}

impl fmt::Debug for CreateUserAuthority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CreateUserAuthority")
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn testing_manual_debug_impl() {
        let create_user_authority = CreateUserAuthority {
            authority_id: uuid::uuid!("97edd536-4c3c-4feb-8a27-efde58cbd21c"),
            user_identifier: "username".to_owned(),
            params: json!({
                "password": "super_secret_password",
            }),
        };

        assert_eq!(format!("{create_user_authority:?}"), "CreateUserAuthority { authority_id: 97edd536-4c3c-4feb-8a27-efde58cbd21c, user_identifier: \"username\" }");
    }
}
