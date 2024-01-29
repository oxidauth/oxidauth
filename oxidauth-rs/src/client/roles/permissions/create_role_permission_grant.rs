use oxidauth_http::{
    response::Response,
    server::api::v1::roles::permissions::create_role_permission_grant::{
        CreateRolePermissionGrantReq, CreateRolePermissionGrantRes,
    },
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::RolePermissionGrant;
const METHOD: &str = "create_role_permission_grant";

impl Client {
    async fn create_role_permission_grant<T>(
        &self,
        role_permission_grant: T,
    ) -> Result<CreateRolePermissionGrantRes, BoxedError>
    where
        T: Into<CreateRolePermissionGrantReq>,
    {
        let role_permission_grant = role_permission_grant.into();

        let resp: Response<CreateRolePermissionGrantRes> = self
            .post(
                &format!(
                    "/roles/{}/permissions/{}",
                    role_permission_grant.role_id,
                    role_permission_grant.permission
                ),
                role_permission_grant,
            )
            .await?;

        let role_permission_grant_res =
            handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_permission_grant_res)
    }
}
