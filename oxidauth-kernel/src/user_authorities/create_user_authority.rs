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
    JsonValue,
};

pub use super::UserAuthority;

pub type CreateUserAuthorityService = Arc<
    dyn for<'a> Service<
        &'a CreateUserAuthorityParams,
        Response = UserAuthority,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserAuthorityParams {
    pub user_id: Uuid,
    pub strategy: AuthorityStrategy,
    pub params: JsonValue,
}

impl From<&CreateUserAuthorityParams> for FindAuthorityByStrategy {
    fn from(value: &CreateUserAuthorityParams) -> Self {
        Self {
            strategy: value.strategy,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateUserAuthority {
    pub authority_id: Uuid,
    pub user_identifier: String,
    pub params: JsonValue,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn testing_manual_json_value_debug_impl() {
        let create_user_authority = CreateUserAuthority {
            authority_id: uuid::uuid!("97edd536-4c3c-4feb-8a27-efde58cbd21c"),
            user_identifier: "username".to_owned(),
            params: JsonValue(json!({
                "password": "super_secret_password",
            })),
        };

        assert_eq!(format!("{create_user_authority:?}"), "CreateUserAuthority { authority_id: 97edd536-4c3c-4feb-8a27-efde58cbd21c, user_identifier: \"username\", params: JsonValue }");
    }
}
