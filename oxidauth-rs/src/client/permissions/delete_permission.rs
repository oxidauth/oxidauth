use oxidauth_http::{
    response::Response,
    server::api::v1::permissions::delete_permission::{DeletePermissionReq, DeletePermissionRes},
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Permission;
const METHOD: &str = "delete_permission";

impl Client {
    async fn delete_permission<T>(
        &self,
        permission: T,
    ) -> Result<DeletePermissionRes, BoxedError>
        where
            T: Into<DeletePermissionReq>,
    {
        let permission = permission.into();

        let resp: Response<DeletePermissionRes> = self
            .post(
                &format!("/permissions/{}", permission.permission),
                None::<DeletePermissionReq>,
            )
            .await?;

        let permission_res = handle_response(
            RESOURCE,
            METHOD,
            resp,
        )?;

        Ok(permission_res)
    }
}
