use async_trait::async_trait;
pub use oxidauth_http::server::api::v1::permissions::find_permission_by_parts::{FindPermissionByPartsReq, FindPermissionByPartsRes};
use oxidauth_http::response::Response;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Permission;
const METHOD: &str = "find_permission_by_parts";

#[async_trait]
pub trait FindPermissionByPartsTrait {
    async fn find_permission_by_parts<T>(
        &self,
        permission: T,
    ) -> Result<FindPermissionByPartsRes, BoxedError>
    where
        T: Into<FindPermissionByPartsReq> + fmt::Debug + Send;
}

#[async_trait]
impl FindPermissionByPartsTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn find_permission_by_parts<T>(
        &self,
        permission: T,
    ) -> Result<FindPermissionByPartsRes, BoxedError>
    where
        T: Into<FindPermissionByPartsReq> + fmt::Debug + Send,
    {
        let permission = permission.into();

        let resp: Response<FindPermissionByPartsRes> = self
            .get(
                &format!(
                    "/permissions/{}",
                    permission.permission
                ),
                None::<FindPermissionByPartsReq>,
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
impl FindPermissionByPartsTrait for ClientMock {
    async fn find_permission_by_parts<T>(
        &self,
        permission: T,
    ) -> Result<FindPermissionByPartsRes, BoxedError>
    where
        T: Into<FindPermissionByPartsReq> + fmt::Debug + Send,
    {
        let Some(func) = self.find_permission_by_parts_fn.clone() else {
            panic!("find_permission_by_parts not defined for mock client");
        };

        return func(permission.into());
    }
}
