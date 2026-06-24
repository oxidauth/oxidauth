use async_trait::async_trait;
pub use oxidauth_http::{
    response::Response,
    server::api::v1::auth::register::{
        AuthorityStrategy,
        RegisterReq,
        RegisterRes,
    },
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Auth;
const METHOD: &str = "register";

#[async_trait]
pub trait RegisterTrait {
    async fn register<T>(&self, params: T) -> Result<RegisterRes, BoxedError>
    where
        T: Into<RegisterReq> + fmt::Debug + Send;
}

#[async_trait]
impl RegisterTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn register<T>(&self, params: T) -> Result<RegisterRes, BoxedError>
    where
        T: Into<RegisterReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<RegisterRes> = self
            .post("/auth/register", params)
            .await?;

        let role_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl RegisterTrait for ClientMock {
    async fn register<T>(&self, params: T) -> Result<RegisterRes, BoxedError>
    where
        T: Into<RegisterReq> + fmt::Debug + Send,
    {
        let Some(func) = self.register_fn.clone() else {
            panic!("register not defined for mock client");
        };

        return func(params.into());
    }
}
