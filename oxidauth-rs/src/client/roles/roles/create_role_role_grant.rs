use oxidauth_http::{
    response::Response,
    server::api::v1::roles::roles::create_role_role_grant::{
        CreateRoleRoleGrantReq, CreateRoleRoleGrantRes,
    },
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::RoleRoleGrant;
const METHOD: &str = "create_role_role_grant";

impl Client {
    async fn create_role_role_grant<T>(
        &self,
        role_role_grant: T,
    ) -> Result<CreateRoleRoleGrantRes, BoxedError>
    where
        T: Into<CreateRoleRoleGrantReq>,
    {
        let role_role_grant = role_role_grant.into();

        let resp: Response<CreateRoleRoleGrantRes> = self
            .post(
                &format!(
                    "/roles/{}/roles/{}",
                    role_role_grant.parent_id, role_role_grant.child_id
                ),
                role_role_grant,
            )
            .await?;

        let role_role_grant_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_role_grant_res)
    }
}
