use oxidauth_http::{
    response::Response,
    server::api::v1::permissions::list_all_permissions::{ListAllPermissionsReq, ListAllPermissionsRes},
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Permission;
const METHOD: &str = "list_all_permissions";

impl Client {
    async fn list_all_permissions<T>(
        &self,
        params: T,
    ) -> Result<ListAllPermissionsRes, BoxedError>
        where
            T: Into<ListAllPermissionsReq>,
    {
        let params = params.into();

        let resp: Response<ListAllPermissionsRes> = self
            .post(
                "/permissions",
                params,
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
