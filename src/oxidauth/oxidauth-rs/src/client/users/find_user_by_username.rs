use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::find_user_by_username::FindUserByUsernameRes;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "find_user_by_username";

#[async_trait]
pub trait FindUserByUsernameTrait {
    async fn find_user_by_username<T>(
        &self,
        username: T,
    ) -> Result<FindUserByUsernameRes, BoxedError>
    where
        T: Into<String> + fmt::Debug + Send;
}

#[async_trait]
impl FindUserByUsernameTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn find_user_by_username<T>(
        &self,
        username: T,
    ) -> Result<FindUserByUsernameRes, BoxedError>
    where
        T: Into<String> + fmt::Debug + Send,
    {
        let username = username.into();

        let resp: Response<FindUserByUsernameRes> = self
            .get(
                &format!(
                    "/users/by_username/{}",
                    username,
                ),
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
impl FindUserByUsernameTrait for ClientMock {
    async fn find_user_by_username<T>(
        &self,
        username: T,
    ) -> Result<FindUserByUsernameRes, BoxedError>
    where
        T: Into<String> + fmt::Debug + Send,
    {
        let Some(func) = self
            .find_user_by_username_fn
            .clone()
        else {
            panic!("find_user_by_username not defined for mock client");
        };

        return func(username.into());
    }
}
