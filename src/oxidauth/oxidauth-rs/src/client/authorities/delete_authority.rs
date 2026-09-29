use uuid::Uuid;
use async_trait::async_trait;

use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::authorities::delete_authority::DeleteAuthorityRes;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Authority;
const METHOD: &str = "delete_authority";

#[async_trait]
pub trait DeleteAuthorityTrait {
    async fn delete_authority<T>(
        &self,
        authority_id: T,
    ) -> Result<DeleteAuthorityRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send;
}

#[async_trait]
impl DeleteAuthorityTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn delete_authority<T>(
        &self,
        authority_id: T,
    ) -> Result<DeleteAuthorityRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
    {
        let authority_id = authority_id.into();

        let resp: Response<DeleteAuthorityRes> = self
            .delete(
                &format!(
                    "/authorities/{}",
                    authority_id
                ),
                None::<()>,
            )
            .await?;

        let authority_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(authority_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl DeleteAuthorityTrait for ClientMock {
    async fn delete_authority<T>(
        &self,
        authority_id: T,
    ) -> Result<DeleteAuthorityRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
    {
        let Some(func) = self
            .delete_authority_fn
            .clone()
        else {
            panic!("delete_authority not defined for mock client");
        };

        return func(authority_id.into());
    }
}
