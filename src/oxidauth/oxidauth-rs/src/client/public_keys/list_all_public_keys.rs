use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::public_keys::list_all_public_keys::ListAllPublicKeysRes;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::PublicKey;
const METHOD: &str = "list_all_public_keys";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ListAllPublicKeysTrait {
    async fn list_all_public_keys(&self) -> Result<ListAllPublicKeysRes, BoxedError>;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl ListAllPublicKeysTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn list_all_public_keys(&self) -> Result<ListAllPublicKeysRes, BoxedError> {
        let resp: Response<ListAllPublicKeysRes> = self
            .get("/public_keys", None::<()>)
            .await?;

        let public_key_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(public_key_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl ListAllPublicKeysTrait for ClientMock {
    async fn list_all_public_keys(&self) -> Result<ListAllPublicKeysRes, BoxedError> {
        let Some(func) = self
            .list_all_public_keys_fn
            .clone()
        else {
            panic!("list_all_public_keys not defined for mock client");
        };

        return func();
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::client::users::contract::{contract, public_key};

    #[tokio::test]
    async fn list_all_public_keys_route_contract() {
        contract(
            "GET",
            "/api/v1/public_keys",
            ("public_key", "list_all_public_keys"),
            json!({ "public_keys": [public_key()] }),
            |client| {
                async move {
                    client
                        .list_all_public_keys()
                        .await
                }
            },
        )
        .await;
    }
}
