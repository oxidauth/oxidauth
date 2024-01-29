use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::roles::roles::delete_role_role_grant::{
    DeleteRoleRoleGrantReq, DeleteRoleRoleGrantRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::RoleRoleGrant;
const METHOD: &str = "delete_role_role_grant";

impl Client {
    pub async fn delete_role_role_grant<T>(
        &self,
        role_role_grant: T,
    ) -> Result<DeleteRoleRoleGrantRes, BoxedError>
    where
        T: Into<DeleteRoleRoleGrantReq>,
    {
        let role_role_grant = role_role_grant.into();

        let resp: Response<DeleteRoleRoleGrantRes> = self
            .delete(
                &format!(
                    "/roles/{}/roles/{}",
                    role_role_grant.parent_id, role_role_grant.child_id
                ),
                None::<DeleteRoleRoleGrantReq>,
            )
            .await?;

        let role_role_grant_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_role_grant_res)
    }
}
