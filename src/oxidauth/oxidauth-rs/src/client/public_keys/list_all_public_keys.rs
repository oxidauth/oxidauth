use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::public_keys::list_all_public_keys::ListAllPublicKeysRes;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::PublicKey;
const METHOD: &str = "list_all_public_keys";

#[async_trait]
pub trait ListAllPublicKeysTrait {
    async fn list_all_public_keys(
        &self,
    ) -> Result<ListAllPublicKeysRes, BoxedError>;
}

#[async_trait]
impl ListAllPublicKeysTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn list_all_public_keys(
        &self,
    ) -> Result<ListAllPublicKeysRes, BoxedError> {
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
#[async_trait]
impl ListAllPublicKeysTrait for ClientMock {
    async fn list_all_public_keys(
        &self,
    ) -> Result<ListAllPublicKeysRes, BoxedError> {
        let Some(func) = self
            .list_all_public_keys_fn
            .clone()
        else {
            panic!("list_all_public_keys not defined for mock client");
        };

        return func();
    }
}
