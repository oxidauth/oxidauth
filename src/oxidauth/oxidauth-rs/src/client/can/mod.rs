use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::can::CanReq;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Permission;
const METHOD: &str = "can";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait CanTrait {
    async fn can<T>(&self, params: T) -> Result<bool, BoxedError>
    where
        T: Into<CanReq> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl CanTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn can<T>(&self, params: T) -> Result<bool, BoxedError>
    where
        T: Into<CanReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<bool> = self
            .get(&format!("/can/{}", params.permission), None::<CanReq>)
            .await?;

        let can_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(can_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl CanTrait for ClientMock {
    async fn can<T>(&self, params: T) -> Result<bool, BoxedError>
    where
        T: Into<CanReq> + fmt::Debug + Send,
    {
        let Some(func) = self.can_fn.clone() else {
            panic!("can not defined for mock client");
        };

        return func(params.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::client::users::contract::contract;

    #[tokio::test]
    async fn can_route_contract() {
        contract(
            "GET",
            "/api/v1/can/oxidauth:users:read",
            ("permission", "can"),
            // the can endpoint payload is a bare bool
            json!(true),
            |client| {
                async move {
                    client
                        .can(CanReq {
                            permission: "oxidauth:users:read".to_string(),
                        })
                        .await
                }
            },
        )
        .await;
    }
}
