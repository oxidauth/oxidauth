use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::roles::permissions::list_role_permission_grants_by_role_id::{ListRolePermissionGrantsByRoleIdReq, ListRolePermissionGrantsByRoleIdRes};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::RolePermissionGrant;
const METHOD: &str = "list_role_permission_grants_by_role_id";

impl Client {
    pub async fn list_role_permission_grants_by_role_id<T>(
        &self,
        params: T,
    ) -> Result<ListRolePermissionGrantsByRoleIdRes, BoxedError>
    where
        T: Into<ListRolePermissionGrantsByRoleIdReq>,
    {
        let params = params.into();

        let resp: Response<ListRolePermissionGrantsByRoleIdRes> = self
            .get(
                &format!(
                    "/roles/{}/permissions",
                    params.role_id
                ),
                None::<ListRolePermissionGrantsByRoleIdReq>,
            )
            .await?;

        let role_permission_grants_res =
            handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_permission_grants_res)
    }
}
