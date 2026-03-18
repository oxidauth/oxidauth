use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::public_keys::create_public_key::CreatePublicKeyRes;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::PublicKey;
const METHOD: &str = "create_public_key";

#[async_trait]
pub trait CreatePublicKeyTrait {
    async fn create_public_key(&self)
    -> Result<CreatePublicKeyRes, BoxedError>;
}

#[async_trait]
impl CreatePublicKeyTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn create_public_key(
        &self,
    ) -> Result<CreatePublicKeyRes, BoxedError> {
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
#[async_trait]
impl CreatePublicKeyTrait for ClientMock {
    async fn create_public_key(
        &self,
    ) -> Result<CreatePublicKeyRes, BoxedError> {
        let Some(func) = self
            .create_public_key_fn
            .clone()
        else {
            panic!("create_public_key not defined for mock client");
        };

        return func();
    }
}
