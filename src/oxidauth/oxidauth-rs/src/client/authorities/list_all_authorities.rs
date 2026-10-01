use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::authorities::list_all_authorities::{
    ListAllAuthoritiesReq,
    ListAllAuthoritiesRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Authority;
const METHOD: &str = "list_all_authorities";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ListAllAuthoritiesTrait {
    async fn list_all_authorities<T>(&self, params: T) -> Result<ListAllAuthoritiesRes, BoxedError>
    where
        T: Into<ListAllAuthoritiesReq> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl ListAllAuthoritiesTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn list_all_authorities<T>(&self, params: T) -> Result<ListAllAuthoritiesRes, BoxedError>
    where
        T: Into<ListAllAuthoritiesReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<ListAllAuthoritiesRes> = self
            .get("/authorities", params)
            .await?;

        let authority_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(authority_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl ListAllAuthoritiesTrait for ClientMock {
    async fn list_all_authorities<T>(&self, params: T) -> Result<ListAllAuthoritiesRes, BoxedError>
    where
        T: Into<ListAllAuthoritiesReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .list_all_authorities_fn
            .clone()
        else {
            panic!("list_all_authorities not defined for mock client");
        };

        return func(params.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::client::users::contract::{authority, contract};

    #[tokio::test]
    async fn list_all_authorities_route_contract() {
        contract(
            "GET",
            "/api/v1/authorities",
            ("authority", "list_all_authorities"),
            json!({ "authorities": [authority(), authority()] }),
            |client| {
                async move {
                    client
                        .list_all_authorities(ListAllAuthoritiesReq {})
                        .await
                }
            },
        )
        .await;
    }
}
