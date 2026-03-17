use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::roles::find_role_by_name::FindRoleByNameRes;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Role;
const METHOD: &str = "find_role_by_name";

#[async_trait]
pub trait FindRoleByNameTrait {
    async fn find_role_by_name<T>(
        &self,
        role: T,
    ) -> Result<FindRoleByNameRes, BoxedError>
    where
        T: Into<String> + fmt::Debug + Send;
}

#[async_trait]
impl FindRoleByNameTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn find_role_by_name<T>(
        &self,
        role: T,
    ) -> Result<FindRoleByNameRes, BoxedError>
    where
        T: Into<String> + fmt::Debug + Send,
    {
        let role = role.into();

        let resp: Response<FindRoleByNameRes> = self
            .get(
                &format!("/roles/by_name/{}", role),
                None::<()>,
            )
            .await?;

        let role_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl FindRoleByNameTrait for ClientMock {
    async fn find_role_by_name<T>(
        &self,
        role: T,
    ) -> Result<FindRoleByNameRes, BoxedError>
    where
        T: Into<String> + fmt::Debug + Send,
    {
        let Some(func) = self.find_role_by_name_fn.clone() else {
            panic!("find_role_by_name not defined for mock client");
        };

        return func(role.into());
    }
}
