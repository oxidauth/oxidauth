use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::find_users_by_ids::{
    FindUsersByIdsReq,
    FindUsersByIdsRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "find_users_by_ids";

#[async_trait]
pub trait FindUsersByIdsTrait {
    async fn find_users_by_ids<T>(
        &self,
        params: T,
    ) -> Result<FindUsersByIdsRes, BoxedError>
    where
        T: Into<FindUsersByIdsReq> + fmt::Debug + Send;
}

#[async_trait]
impl FindUsersByIdsTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn find_users_by_ids<T>(
        &self,
        params: T,
    ) -> Result<FindUsersByIdsRes, BoxedError>
    where
        T: Into<FindUsersByIdsReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<FindUsersByIdsRes> = self
            .post("/users/by_ids", Some(params))
            .await?;

        let users_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(users_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl FindUsersByIdsTrait for ClientMock {
    async fn find_users_by_ids<T>(
        &self,
        params: T,
    ) -> Result<FindUsersByIdsRes, BoxedError>
    where
        T: Into<FindUsersByIdsReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .find_users_by_ids_fn
            .clone()
        else {
            panic!("find_users_by_ids not defined for mock client");
        };

        return func(params.into());
    }
}
