use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::authorities::create_authority::{
    CreateAuthority, CreateAuthorityReq, CreateAuthorityRes,
};
pub use oxidauth_kernel::authorities::TotpSettings;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Authority;
const METHOD: &str = "create_authority";

#[async_trait]
pub trait CreateAuthorityTrait {
    async fn create_authority<T>(
        &self,
        authority: T,
    ) -> Result<CreateAuthorityRes, BoxedError>
    where
        T: Into<CreateAuthorityReq> + fmt::Debug + Send;
}

#[async_trait]
impl CreateAuthorityTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn create_authority<T>(
        &self,
        authority: T,
    ) -> Result<CreateAuthorityRes, BoxedError>
    where
        T: Into<CreateAuthorityReq> + fmt::Debug + Send,
    {
        let authority = authority.into();

        let resp: Response<CreateAuthorityRes> = self
            .post("/authorities", authority)
            .await?;

        let authority_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(authority_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl CreateAuthorityTrait for ClientMock {
    async fn create_authority<T>(
        &self,
        authority: T,
    ) -> Result<CreateAuthorityRes, BoxedError>
    where
        T: Into<CreateAuthorityReq> + fmt::Debug + Send,
    {
        let Some(func) = self.create_authority_fn.clone() else {
            panic!("create_authority not defined for mock client");
        };

        return func(authority.into());
    }
}
