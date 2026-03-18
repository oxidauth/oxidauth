use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::roles::create_user_role::CreateUserRoleRes;
use oxidauth_kernel::error::BoxedError;
use serde::Serialize;
use uuid::Uuid;

use super::*;

#[derive(Debug, Serialize)]
pub struct CreateUserRole {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

const RESOURCE: Resource = Resource::UserRole;
const METHOD: &str = "create_user_role";

#[async_trait]
pub trait CreateUserRoleTrait {
    async fn create_user_role<T>(
        &self,
        params: T,
    ) -> Result<CreateUserRoleRes, BoxedError>
    where
        T: Into<CreateUserRole> + fmt::Debug + Send;
}

#[async_trait]
impl CreateUserRoleTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn create_user_role<T>(
        &self,
        params: T,
    ) -> Result<CreateUserRoleRes, BoxedError>
    where
        T: Into<CreateUserRole> + fmt::Debug + Send,
    {
        let CreateUserRole { user_id, role_id } = params.into();

        let resp: Response<CreateUserRoleRes> = self
            .post(
                &format!(
                    "/users/{}/roles/{}",
                    user_id, role_id
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
impl CreateUserRoleTrait for ClientMock {
    async fn create_user_role<T>(
        &self,
        params: T,
    ) -> Result<CreateUserRoleRes, BoxedError>
    where
        T: Into<CreateUserRole> + fmt::Debug + Send,
    {
        let Some(func) = self
            .create_user_role_fn
            .clone()
        else {
            panic!("create_user_role not defined for mock client");
        };

        return func(params.into());
    }
}
