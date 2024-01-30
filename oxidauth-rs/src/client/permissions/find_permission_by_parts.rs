use oxidauth_http::{
    response::Response,
    server::api::v1::permissions::find_permission_by_parts::{FindPermissionByPartsReq, FindPermissionByPartsRes},
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Permission;
const METHOD: &str = "find_permission_by_parts";

impl Client {
    pub async fn find_permission_by_parts<T>(
        &self,
        permission: T,
    ) -> Result<FindPermissionByPartsRes, BoxedError>
        where
            T: Into<FindPermissionByPartsReq>,
    {
        let permission = permission.into();

        let resp: Response<FindPermissionByPartsRes> = self
            .get(
                &format!("/permissions/{}", permission.permission),
                None::<FindPermissionByPartsReq>,
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
