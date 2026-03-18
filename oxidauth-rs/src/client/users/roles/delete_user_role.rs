use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::roles::delete_user_role::DeleteUserRoleRes;
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::UserRole;
const METHOD: &str = "delete_user_role";

#[async_trait]
pub trait DeleteUserRoleTrait {
    async fn delete_user_role<T, R>(
        &self,
        user_id: T,
        role_id: R,
    ) -> Result<DeleteUserRoleRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
        R: Into<Uuid> + fmt::Debug + Send;
}

#[async_trait]
impl DeleteUserRoleTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn delete_user_role<T, R>(
        &self,
        user_id: T,
        role_id: R,
    ) -> Result<DeleteUserRoleRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
        R: Into<Uuid> + fmt::Debug + Send,
    {
        let user_id = user_id.into();
        let role_id = role_id.into();

        let resp: Response<DeleteUserRoleRes> = self
            .delete(
                &format!(
                    "/users/{}/roles/{}",
                    user_id, role_id
                ),
                None::<()>,
            )
            .await?;

        let user_role_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_role_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl DeleteUserRoleTrait for ClientMock {
    async fn delete_user_role<T, R>(
        &self,
        user_id: T,
        role_id: R,
    ) -> Result<DeleteUserRoleRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
        R: Into<Uuid> + fmt::Debug + Send,
    {
        let Some(func) = self
            .delete_user_role_fn
            .clone()
        else {
            panic!("delete_user_role not defined for mock client");
        };

        return func(user_id.into(), role_id.into());
    }
}
