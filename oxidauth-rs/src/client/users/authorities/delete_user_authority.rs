use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::authorities::delete_user_authority::{
    DeleteUserAuthorityReq, DeleteUserAuthorityRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::UserAuthority;
const METHOD: &str = "delete_user_authority";

#[async_trait]
pub trait DeleteUserAuthorityTrait {
    async fn delete_user_authority<T>(
        &self,
        params: T,
    ) -> Result<DeleteUserAuthorityRes, BoxedError>
    where
        T: Into<DeleteUserAuthorityReq> + fmt::Debug + Send;
}

#[async_trait]
impl DeleteUserAuthorityTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn delete_user_authority<T>(
        &self,
        params: T,
    ) -> Result<DeleteUserAuthorityRes, BoxedError>
    where
        T: Into<DeleteUserAuthorityReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<DeleteUserAuthorityRes> = self
            .delete(
                &format!(
                    "/users/{}/authorities/{}",
                    params.user_id, params.authority_id
                ),
                None::<DeleteUserAuthorityReq>,
            )
            .await?;

        let user_authority_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_authority_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl DeleteUserAuthorityTrait for ClientMock {
    async fn delete_user_authority<T>(
        &self,
        params: T,
    ) -> Result<DeleteUserAuthorityRes, BoxedError>
    where
        T: Into<DeleteUserAuthorityReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .delete_user_authority_fn
            .clone()
        else {
            panic!("delete_user_authority not defined for mock client");
        };

        return func(params.into());
    }
}
