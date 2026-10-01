use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::users::find_user_by_username::FindUserByUsernameRes;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "find_user_by_username";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait FindUserByUsernameTrait {
    async fn find_user_by_username<T>(
        &self,
        username: T,
    ) -> Result<FindUserByUsernameRes, BoxedError>
    where
        T: Into<String> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl FindUserByUsernameTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn find_user_by_username<T>(
        &self,
        username: T,
    ) -> Result<FindUserByUsernameRes, BoxedError>
    where
        T: Into<String> + fmt::Debug + Send,
    {
        let username = username.into();

        let resp: Response<FindUserByUsernameRes> = self
            .get(&format!("/users/by_username/{}", username,), None::<()>)
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
impl FindUserByUsernameTrait for ClientMock {
    async fn find_user_by_username<T>(
        &self,
        username: T,
    ) -> Result<FindUserByUsernameRes, BoxedError>
    where
        T: Into<String> + fmt::Debug + Send,
    {
        let Some(func) = self
            .find_user_by_username_fn
            .clone()
        else {
            panic!("find_user_by_username not defined for mock client");
        };

        return func(username.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::client::users::contract::{contract, user};

    #[tokio::test]
    async fn find_user_by_username_route_contract() {
        contract(
            "GET",
            "/api/v1/users/by_username/malreynolds",
            ("user", "find_user_by_username"),
            json!({ "user": user() }),
            |client| {
                async move {
                    client
                        .find_user_by_username("malreynolds".to_string())
                        .await
                }
            },
        )
        .await;
    }
}
