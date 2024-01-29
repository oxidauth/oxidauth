use oxidauth_http::{
    response::Response,
    server::api::v1::users::permissions::list_user_permission_grants_by_user_id::{ListUserPermissionGrantsByUserIdReq, ListUserPermissionGrantsByUserIdRes},
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::UserPermissionGrant;
const METHOD: &str = "list_user_permission_grants_by_user_id";

impl Client {
    async fn list_user_permission_grants_by_user_id<T>(
        &self,
        params: T,
    ) -> Result<ListUserPermissionGrantsByUserIdRes, BoxedError>
    where
        T: Into<ListUserPermissionGrantsByUserIdReq>,
    {
        let params = params.into();

        let resp: Response<ListUserPermissionGrantsByUserIdRes> = self
            .get(
                &format!(
                    "/users/{}/permissions",
                    params.user_id
                ),
                None::<()>,
            )
            .await?;

        let user_permission_grants_res =
            handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_permission_grants_res)
    }
}
