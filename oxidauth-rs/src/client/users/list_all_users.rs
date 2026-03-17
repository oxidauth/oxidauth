use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::list_all_users::{
    ListAllUsersReq, ListAllUsersRes,
};
use oxidauth_kernel::error::BoxedError;
use async_trait::async_trait;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "list_all_users";

#[async_trait]
pub trait ListAllUsersTrait {
    async fn list_all_users<T>(
        &self,
        params: T,
    ) -> Result<ListAllUsersRes, BoxedError>
    where
        T: Into<ListAllUsersReq> + fmt::Debug + Send;
}

#[async_trait]
impl ListAllUsersTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn list_all_users<T>(
        &self,
        params: T,
    ) -> Result<ListAllUsersRes, BoxedError>
    where
        T: Into<ListAllUsersReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<ListAllUsersRes> = self
            .get("/users", params)
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl ListAllUsersTrait for ClientMock {
    async fn list_all_users<T>(
        &self,
        params: T,
    ) -> Result<ListAllUsersRes, BoxedError>
    where
        T: Into<ListAllUsersReq> + fmt::Debug + Send {
            let Some(func) = self.list_all_users_fn.clone() else {
                panic!("list_all_users not defined for mock client");
            };

            return func(params.into())
        }
}

