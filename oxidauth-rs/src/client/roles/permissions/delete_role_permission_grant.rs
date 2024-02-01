use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::roles::permissions::delete_role_permission_grant::{
    DeleteRolePermissionGrantReq, DeleteRolePermissionGrantRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::RolePermissionGrant;
const METHOD: &str = "delete_role_permission_grant";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn delete_role_permission_grant<T>(
        &self,
        role_permission_grant: T,
    ) -> Result<DeleteRolePermissionGrantRes, BoxedError>
    where
        T: Into<DeleteRolePermissionGrantReq> + fmt::Debug,
    {
        let role_permission_grant = role_permission_grant.into();

        let resp: Response<DeleteRolePermissionGrantRes> = self
            .delete(
                &format!(
                    "/roles/{}/permissions/{}",
                    role_permission_grant.role_id,
                    role_permission_grant.permission
                ),
                None::<DeleteRolePermissionGrantReq>,
            )
            .await?;

        let role_permission_grant_res =
            handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_permission_grant_res)
    }
}
