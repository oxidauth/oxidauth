use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::users::list_all_users::{ListAllUsersReq, ListAllUsersRes};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "list_all_users";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ListAllUsersTrait {
    async fn list_all_users<T>(&self, params: T) -> Result<ListAllUsersRes, BoxedError>
    where
        T: Into<ListAllUsersReq> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl ListAllUsersTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn list_all_users<T>(&self, params: T) -> Result<ListAllUsersRes, BoxedError>
    where
        T: Into<ListAllUsersReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<ListAllUsersRes> = self
            .get("/users", params)
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
impl ListAllUsersTrait for ClientMock {
    async fn list_all_users<T>(&self, params: T) -> Result<ListAllUsersRes, BoxedError>
    where
        T: Into<ListAllUsersReq> + fmt::Debug + Send,
    {
        let Some(func) = self.list_all_users_fn.clone() else {
            panic!("list_all_users not defined for mock client");
        };

        return func(params.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::client::users::contract::{contract, user};

    #[tokio::test]
    async fn list_all_users_route_contract() {
        contract(
            "GET",
            "/api/v1/users",
            ("user", "list_all_users"),
            json!({ "users": [user(), user()] }),
            |client| {
                async move {
                    client
                        .list_all_users(ListAllUsersReq {})
                        .await
                }
            },
        )
        .await;
    }
}
