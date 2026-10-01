use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::public_keys::create_public_key::CreatePublicKeyRes;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::PublicKey;
const METHOD: &str = "create_public_key";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait CreatePublicKeyTrait {
    async fn create_public_key(&self) -> Result<CreatePublicKeyRes, BoxedError>;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl CreatePublicKeyTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn create_public_key(&self) -> Result<CreatePublicKeyRes, BoxedError> {
        let resp: Response<CreatePublicKeyRes> = self
            .post("/public_keys", None::<()>)
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
impl CreatePublicKeyTrait for ClientMock {
    async fn create_public_key(&self) -> Result<CreatePublicKeyRes, BoxedError> {
        let Some(func) = self
            .create_public_key_fn
            .clone()
        else {
            panic!("create_public_key not defined for mock client");
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
    async fn create_public_key_route_contract() {
        contract(
            "POST",
            "/api/v1/public_keys",
            ("public_key", "create_public_key"),
            json!({ "public_key": public_key() }),
            |client| {
                async move {
                    client
                        .create_public_key()
                        .await
                }
            },
        )
        .await;
    }
}
