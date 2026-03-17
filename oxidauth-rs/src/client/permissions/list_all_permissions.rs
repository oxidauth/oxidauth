use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::permissions::list_all_permissions::{
    ListAllPermissionsReq, ListAllPermissionsRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Permission;
const METHOD: &str = "list_all_permissions";

#[async_trait]
pub trait ListAllPermissionsTrait {
    async fn list_all_permissions<T>(
        &self,
        params: T,
    ) -> Result<ListAllPermissionsRes, BoxedError>
    where
        T: Into<ListAllPermissionsReq> + fmt::Debug + Send;
}

#[async_trait]
impl ListAllPermissionsTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn list_all_permissions<T>(
        &self,
        params: T,
    ) -> Result<ListAllPermissionsRes, BoxedError>
    where
        T: Into<ListAllPermissionsReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<ListAllPermissionsRes> = self
            .get("/permissions", params)
            .await?;

        let permission_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(permission_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl ListAllPermissionsTrait for ClientMock {
    async fn list_all_permissions<T>(
        &self,
        params: T,
    ) -> Result<ListAllPermissionsRes, BoxedError>
    where
        T: Into<ListAllPermissionsReq> + fmt::Debug + Send,
    {
        let Some(func) = self.list_all_permissions_fn.clone() else {
            panic!("list_all_permissions not defined for mock client");
        };

        return func(params.into());
    }
}
