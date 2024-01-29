use oxidauth_http::{
    response::Response,
    server::api::v1::roles::roles::list_role_role_grants_by_parent_id::{
        ListRoleRoleGrantsByParentIdReq, ListRoleRoleGrantsByParentIdRes,
    },
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::RoleRoleGrant;
const METHOD: &str = "list_role_role_grants_by_parent_id";

impl Client {
    async fn list_role_role_grants_by_parent_id<T>(
        &self,
        params: T,
    ) -> Result<ListRoleRoleGrantsByParentIdRes, BoxedError>
    where
        T: Into<ListRoleRoleGrantsByParentIdReq>,
    {
        let params = params.into();

        let resp: Response<ListRoleRoleGrantsByParentIdRes> = self
            .get(
                &format!(
                    "/roles/{}/roles",
                    params.parent_id
                ),
                params,
            )
            .await?;

        let role_role_grants_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_role_grants_res)
    }
}
