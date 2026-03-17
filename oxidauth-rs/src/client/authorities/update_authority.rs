use uuid::Uuid;
use async_trait::async_trait;

use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::authorities::update_authority::{
    UpdateAuthority, UpdateAuthorityPathReq, UpdateAuthorityReq,
    UpdateAuthorityRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Authority;
const METHOD: &str = "update_authority";

#[async_trait]
pub trait UpdateAuthorityTrait {
    async fn update_authority<T, U>(
        &self,
        authority_id: U,
        params: T,
    ) -> Result<UpdateAuthorityRes, BoxedError>
    where
        U: Into<Uuid> + fmt::Debug + Send,
        T: Into<UpdateAuthorityReq> + fmt::Debug + Send;
}

#[async_trait]
impl UpdateAuthorityTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn update_authority<T, U>(
        &self,
        authority_id: U,
        params: T,
    ) -> Result<UpdateAuthorityRes, BoxedError>
    where
        U: Into<Uuid> + fmt::Debug + Send,
        T: Into<UpdateAuthorityReq> + fmt::Debug + Send,
    {
        let authority_id = authority_id.into();
        let params = params.into();

        let resp: Response<UpdateAuthorityRes> = self
            .put(
                &format!(
                    "/authorities/{}",
                    authority_id
                ),
                params,
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
impl UpdateAuthorityTrait for ClientMock {
    async fn update_authority<T, U>(
        &self,
        authority_id: U,
        params: T,
    ) -> Result<UpdateAuthorityRes, BoxedError>
    where
        U: Into<Uuid> + fmt::Debug + Send,
        T: Into<UpdateAuthorityReq> + fmt::Debug + Send,
    {
        let Some(func) = self.update_authority_fn.clone() else {
            panic!("update_authority not defined for mock client");
        };

        return func(authority_id.into(), params.into());
    }
}
