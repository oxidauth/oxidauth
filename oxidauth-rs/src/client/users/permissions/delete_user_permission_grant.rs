use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::permissions::delete_user_permission::{
    DeleteUserPermissionReq, DeleteUserPermissionRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::UserPermissionGrant;
const METHOD: &str = "delete_user_permission_grant";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn delete_user_permission_grant<T>(
        &self,
        user_permission_grant: T,
    ) -> Result<DeleteUserPermissionRes, BoxedError>
    where
        T: Into<DeleteUserPermissionReq> + fmt::Debug,
    {
        let user_permission_grant = user_permission_grant.into();

        let resp: Response<DeleteUserPermissionRes> = self
            .delete(
                &format!(
                    "/users/{}/permissions/{}",
                    user_permission_grant.user_id,
                    user_permission_grant.permission
                ),
                None::<()>,
            )
            .await?;

        let user_permission_grant_res =
            handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_permission_grant_res)
    }
}
