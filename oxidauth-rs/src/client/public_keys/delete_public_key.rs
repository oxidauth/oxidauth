use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::public_keys::delete_public_key::DeletePublicKeyRes;
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::PublicKey;
const METHOD: &str = "delete_public_key";

#[async_trait]
pub trait DeletePublicKeyTrait {
    async fn delete_public_key<T>(
        &self,
        public_key_id: T,
    ) -> Result<DeletePublicKeyRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send;
}

#[async_trait]
impl DeletePublicKeyTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn delete_public_key<T>(
        &self,
        public_key_id: T,
    ) -> Result<DeletePublicKeyRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
    {
        let public_key_id = public_key_id.into();

        let resp: Response<DeletePublicKeyRes> = self
            .delete(
                &format!(
                    "/public_keys/{}",
                    public_key_id
                ),
                None::<Uuid>,
            )
            .await?;

        let public_key_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(public_key_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl DeletePublicKeyTrait for ClientMock {
    async fn delete_public_key<T>(
        &self,
        public_key_id: T,
    ) -> Result<DeletePublicKeyRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
    {
        let Some(func) = self.delete_public_key_fn.clone() else {
            panic!("delete_public_key not defined for mock client");
        };

        return func(public_key_id.into());
    }
}
