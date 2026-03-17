use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::public_keys::find_public_key_by_id::FindPublicKeyByIdRes;
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::PublicKey;
const METHOD: &str = "find_public_key_by_id";

#[async_trait]
pub trait FindPublicKeyByIdTrait {
    async fn find_public_key_by_id<T>(
        &self,
        public_key_id: T,
    ) -> Result<FindPublicKeyByIdRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send;
}

#[async_trait]
impl FindPublicKeyByIdTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn find_public_key_by_id<T>(
        &self,
        public_key_id: T,
    ) -> Result<FindPublicKeyByIdRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
    {
        let public_key_id = public_key_id.into();

        let resp: Response<FindPublicKeyByIdRes> = self
            .get(
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
impl FindPublicKeyByIdTrait for ClientMock {
    async fn find_public_key_by_id<T>(
        &self,
        public_key_id: T,
    ) -> Result<FindPublicKeyByIdRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
    {
        let Some(func) = self.find_public_key_by_id_fn.clone() else {
            panic!("find_public_key_by_id not defined for mock client");
        };

        return func(public_key_id.into());
    }
}
