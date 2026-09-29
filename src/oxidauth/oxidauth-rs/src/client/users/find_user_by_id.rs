use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::find_user_by_id::{
    FindUserByIdReq,
    FindUserByIdRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "find_user_by_id";

#[async_trait]
pub trait FindUserByIdTrait {
    async fn find_user_by_id<T>(
        &self,
        params: T,
    ) -> Result<FindUserByIdRes, BoxedError>
    where
        T: Into<FindUserByIdReq> + fmt::Debug + Send;
}

#[async_trait]
impl FindUserByIdTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn find_user_by_id<T>(
        &self,
        params: T,
    ) -> Result<FindUserByIdRes, BoxedError>
    where
        T: Into<FindUserByIdReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<FindUserByIdRes> = self
            .get(
                &format!("/users/{}", params.user_id),
                None::<()>,
            )
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl FindUserByIdTrait for ClientMock {
    async fn find_user_by_id<T>(
        &self,
        params: T,
    ) -> Result<FindUserByIdRes, BoxedError>
    where
        T: Into<FindUserByIdReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .find_user_by_id_fn
            .clone()
        else {
            panic!("find_user_by_id not defined for mock client");
        };

        return func(params.into());
    }
}
