use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::permissions::create_permission::{
    CreatePermissionReq, CreatePermissionRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Permission;
const METHOD: &str = "create_permission";

impl Client {
    pub async fn create_permission<T>(
        &self,
        permission: T,
    ) -> Result<CreatePermissionRes, BoxedError>
    where
        T: Into<CreatePermissionReq>,
    {
        let permission = permission.into();

        let resp: Response<CreatePermissionRes> = self
            .post(
                &format!(
                    "/permissions/{}",
                    permission.permission
                ),
                None::<CreatePermissionReq>,
            )
            .await?;

        let permission_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(permission_res)
    }
}
