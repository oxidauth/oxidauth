use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::users::find_user_by_id::{FindUserByIdReq, FindUserByIdRes};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "find_user_by_id";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait FindUserByIdTrait {
    async fn find_user_by_id<T>(&self, params: T) -> Result<FindUserByIdRes, BoxedError>
    where
        T: Into<FindUserByIdReq> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl FindUserByIdTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn find_user_by_id<T>(&self, params: T) -> Result<FindUserByIdRes, BoxedError>
    where
        T: Into<FindUserByIdReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<FindUserByIdRes> = self
            .get(&format!("/users/{}", params.user_id), None::<()>)
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl FindUserByIdTrait for ClientMock {
    async fn find_user_by_id<T>(&self, params: T) -> Result<FindUserByIdRes, BoxedError>
    where
        T: Into<FindUserByIdReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .find_user_by_id_fn
            .clone()
        else {
            panic!("find_user_by_id not defined for mock client");
        };

        return func(params.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, user};

    #[tokio::test]
    async fn find_user_by_id_route_contract() {
        let user_id = Uuid::new_v4();

        contract(
            "GET",
            &format!("/api/v1/users/{user_id}"),
            ("user", "find_user_by_id"),
            json!({ "user": user() }),
            move |client| {
                async move {
                    client
                        .find_user_by_id(FindUserByIdReq { user_id })
                        .await
                }
            },
        )
        .await;
    }
}
