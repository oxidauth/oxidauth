use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::permissions::delete_permission::{
    DeletePermissionReq, DeletePermissionRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Permission;
const METHOD: &str = "delete_permission";

#[async_trait]
pub trait DeletePermissionTrait {
    async fn delete_permission<T>(
        &self,
        permission: T,
    ) -> Result<DeletePermissionRes, BoxedError>
    where
        T: Into<DeletePermissionReq> + fmt::Debug + Send;
}

#[async_trait]
impl DeletePermissionTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn delete_permission<T>(
        &self,
        permission: T,
    ) -> Result<DeletePermissionRes, BoxedError>
    where
        T: Into<DeletePermissionReq> + fmt::Debug + Send,
    {
        let permission = permission.into();

        let resp: Response<DeletePermissionRes> = self
            .delete(
                &format!(
                    "/permissions/{}",
                    permission.permission
                ),
                None::<DeletePermissionReq>,
            )
            .await?;

        let permission_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(permission_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl DeletePermissionTrait for ClientMock {
    async fn delete_permission<T>(
        &self,
        permission: T,
    ) -> Result<DeletePermissionRes, BoxedError>
    where
        T: Into<DeletePermissionReq> + fmt::Debug + Send,
    {
        let Some(func) = self.delete_permission_fn.clone() else {
            panic!("delete_permission not defined for mock client");
        };

        return func(permission.into());
    }
}
