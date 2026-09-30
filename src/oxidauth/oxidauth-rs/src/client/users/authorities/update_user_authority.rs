use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::users::authorities::update_user_authority::UpdateUserAuthorityRes;
use oxidauth_kernel::{
    error::BoxedError,
    user_authorities::update_user_authority::UpdateUserAuthority,
};

use super::*;

const RESOURCE: Resource = Resource::UserAuthority;
const METHOD: &str = "update_user_authority";

#[async_trait]
pub trait UpdateUserAuthorityTrait {
    async fn update_user_authority<T>(
        &self,
        params: T,
    ) -> Result<UpdateUserAuthorityRes, BoxedError>
    where
        T: Into<UpdateUserAuthority> + fmt::Debug + Send;
}

#[async_trait]
impl UpdateUserAuthorityTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn update_user_authority<T>(
        &self,
        params: T,
    ) -> Result<UpdateUserAuthorityRes, BoxedError>
    where
        T: Into<UpdateUserAuthority> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<UpdateUserAuthorityRes> = self
            .put(
                &format!(
                    "/users/{}/authorities/{}",
                    params.user_id, params.authority_id
                ),
                params,
            )
            .await?;

        let user_authority_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_authority_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl UpdateUserAuthorityTrait for ClientMock {
    async fn update_user_authority<T>(
        &self,
        params: T,
    ) -> Result<UpdateUserAuthorityRes, BoxedError>
    where
        T: Into<UpdateUserAuthority> + fmt::Debug + Send,
    {
        let Some(func) = self
            .update_user_authority_fn
            .clone()
        else {
            panic!("update_user_authority not defined for mock client");
        };

        return func(params.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use oxidauth_kernel::JsonValue;
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, user_authority};

    #[tokio::test]
    async fn update_user_authority_route_contract() {
        let user_id = Uuid::new_v4();
        let authority_id = Uuid::new_v4();

        contract(
            "PUT",
            &format!("/api/v1/users/{user_id}/authorities/{authority_id}"),
            ("user_authority", "update_user_authority"),
            json!({ "user_authority": user_authority() }),
            move |client| {
                async move {
                    client
                        .update_user_authority(UpdateUserAuthority {
                            user_id,
                            authority_id,
                            params: JsonValue::new(json!({ "password": "rotated" })),
                        })
                        .await
                }
            },
        )
        .await;
    }
}
