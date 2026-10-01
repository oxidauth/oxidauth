use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::users::authorities::create_user_authority::{
    CreateUserAuthorityBodyReq,
    CreateUserAuthorityRes,
    UserAuthorityParams,
};
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::UserAuthority;
const METHOD: &str = "create_user_authority";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait CreateUserAuthorityTrait {
    async fn create_user_authority<T, U>(
        &self,
        user_id: T,
        user_authority: U,
    ) -> Result<CreateUserAuthorityRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
        U: Into<CreateUserAuthorityBodyReq> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl CreateUserAuthorityTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn create_user_authority<T, U>(
        &self,
        user_id: T,
        user_authority: U,
    ) -> Result<CreateUserAuthorityRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
        U: Into<CreateUserAuthorityBodyReq> + fmt::Debug + Send,
    {
        let user_id = user_id.into();
        let user_authority = user_authority.into();

        let resp: Response<CreateUserAuthorityRes> = self
            .post(&format!("/users/{}/authorities", user_id), user_authority)
            .await?;

        let user_authority_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_authority_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl CreateUserAuthorityTrait for ClientMock {
    async fn create_user_authority<T, U>(
        &self,
        user_id: T,
        user_authority: U,
    ) -> Result<CreateUserAuthorityRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
        U: Into<CreateUserAuthorityBodyReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .create_user_authority_fn
            .clone()
        else {
            panic!("create_user_authority not defined for mock client");
        };

        return func(user_id.into(), user_authority.into());
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
    async fn create_user_authority_route_contract() {
        let user_id = Uuid::new_v4();

        contract(
            "POST",
            &format!("/api/v1/users/{user_id}/authorities"),
            ("user_authority", "create_user_authority"),
            json!({ "user_authority": user_authority() }),
            move |client| {
                async move {
                    client
                        .create_user_authority(
                            user_id,
                            CreateUserAuthorityBodyReq {
                                client_key: Uuid::new_v4(),
                                user_authority: UserAuthorityParams {
                                    params: JsonValue::new(json!({ "password": "hunter2" })),
                                },
                            },
                        )
                        .await
                }
            },
        )
        .await;
    }
}
