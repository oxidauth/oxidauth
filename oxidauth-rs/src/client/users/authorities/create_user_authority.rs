use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::authorities::create_user_authority::{
    CreateUserAuthorityBodyReq, CreateUserAuthorityRes, UserAuthorityParams,
};
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::UserAuthority;
const METHOD: &str = "create_user_authority";

#[async_trait]
pub trait CreateUserAuthorityTrait {
    async fn create_user_authority<T, U>(
        &self,
        user_id: T,
        user_authority: U,
    ) -> Result<CreateUserAuthorityRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
        U: Into<CreateUserAuthorityBodyReq> + fmt::Debug + Send;
}

#[async_trait]
impl CreateUserAuthorityTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn create_user_authority<T, U>(
        &self,
        user_id: T,
        user_authority: U,
    ) -> Result<CreateUserAuthorityRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
        U: Into<CreateUserAuthorityBodyReq> + fmt::Debug + Send,
    {
        let user_id = user_id.into();
        let user_authority = user_authority.into();

        let resp: Response<CreateUserAuthorityRes> = self
            .post(
                &format!(
                    "/users/{}/authorities",
                    user_id
                ),
                user_authority,
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
impl CreateUserAuthorityTrait for ClientMock {
    async fn create_user_authority<T, U>(
        &self,
        user_id: T,
        user_authority: U,
    ) -> Result<CreateUserAuthorityRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
        U: Into<CreateUserAuthorityBodyReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .create_user_authority_fn
            .clone()
        else {
            panic!("create_user_authority not defined for mock client");
        };

        return func(
            user_id.into(),
            user_authority.into(),
        );
    }
}
