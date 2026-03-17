use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::roles::list_all_roles::{
    ListAllRolesReq, ListAllRolesRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Role;
const METHOD: &str = "list_all_roles";

#[async_trait]
pub trait ListAllRolesTrait {
    async fn list_all_roles<T>(
        &self,
        params: T,
    ) -> Result<ListAllRolesRes, BoxedError>
    where
        T: Into<ListAllRolesReq> + fmt::Debug + Send;
}

#[async_trait]
impl ListAllRolesTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn list_all_roles<T>(
        &self,
        params: T,
    ) -> Result<ListAllRolesRes, BoxedError>
    where
        T: Into<ListAllRolesReq> + fmt::Debug + Send,
    {
        let _params = params.into();

        let resp: Response<ListAllRolesRes> = self
            .get(
                "/roles",
                None::<ListAllRolesReq>,
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
impl ListAllRolesTrait for ClientMock {
    async fn list_all_roles<T>(
        &self,
        params: T,
    ) -> Result<ListAllRolesRes, BoxedError>
    where
        T: Into<ListAllRolesReq> + fmt::Debug + Send,
    {
        let Some(func) = self.list_all_roles_fn.clone() else {
            panic!("list_all_roles not defined for mock client");
        };

        return func(params.into());
    }
}
