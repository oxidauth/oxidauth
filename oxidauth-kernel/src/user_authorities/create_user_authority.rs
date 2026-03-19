use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    authorities::find_authority_by_client_key::FindAuthorityByClientKey,
    error::BoxedError, JsonValue,
};

pub use super::UserAuthority;

#[async_trait]
pub trait CreateUserAuthorityTrait: Send + Sync + 'static {
    async fn create_user_authority(
        &self,
        params: &CreateUserAuthorityParams,
    ) -> Result<UserAuthority, BoxedError>;
}

pub type CreateUserAuthorityService = Arc<dyn CreateUserAuthorityTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserAuthorityParams {
    pub user_id: Uuid,
    pub client_key: Uuid,
    pub params: JsonValue,
}

impl From<&CreateUserAuthorityParams> for FindAuthorityByClientKey {
    fn from(value: &CreateUserAuthorityParams) -> Self {
        Self {
            client_key: value.client_key,
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
