use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::create_user::{
    CreateUserReq,
    CreateUserRes,
    UserKind,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "create_user";

#[async_trait]
pub trait CreateUserTrait {
    async fn create_user<T>(
        &self,
        user: T,
    ) -> Result<CreateUserRes, BoxedError>
    where
        T: Into<CreateUserReq> + fmt::Debug + Send;
}

#[async_trait]
impl CreateUserTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn create_user<T>(&self, user: T) -> Result<CreateUserRes, BoxedError>
    where
        T: Into<CreateUserReq> + fmt::Debug + Send,
    {
        let user = user.into();

        let resp: Response<CreateUserRes> = self
            .post("/users", user)
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl CreateUserTrait for ClientMock {
    async fn create_user<T>(&self, user: T) -> Result<CreateUserRes, BoxedError>
    where
        T: Into<CreateUserReq> + fmt::Debug + Send,
    {
        let Some(func) = self.create_user_fn.clone() else {
            panic!("create_user not defined for mock client");
        };

        return func(user.into());
    }
}
