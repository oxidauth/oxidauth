use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::permissions::delete_permission::{
    DeletePermissionReq, DeletePermissionRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Permission;
const METHOD: &str = "delete_permission";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn delete_permission<T>(
        &self,
        permission: T,
    ) -> Result<DeletePermissionRes, BoxedError>
    where
        T: Into<DeletePermissionReq> + fmt::Debug,
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
