use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::delete_user_by_id::DeleteUserByIdRes;
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "delete_user";

#[async_trait]
pub trait DeleteUserTrait {
    async fn delete_user<T>(
        &self,
        user_id: T,
    ) -> Result<DeleteUserByIdRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send;
}

#[async_trait]
impl DeleteUserTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn delete_user<T>(
        &self,
        user_id: T,
    ) -> Result<DeleteUserByIdRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
    {
        let user_id = user_id.into();

        let resp: Response<DeleteUserByIdRes> = self
            .delete(
                &format!("/users/{}", user_id),
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
impl DeleteUserTrait for ClientMock {
    async fn delete_user<T>(
        &self,
        user_id: T,
    ) -> Result<DeleteUserByIdRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
    {
        let Some(func) = self.delete_user_fn.clone() else {
            panic!("delete_user not defined for mock client");
        };

        return func(user_id.into());
    }
}
