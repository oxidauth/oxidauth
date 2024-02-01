use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::permissions::create_user_permission::{
    CreateUserPermissionReq, CreateUserPermissionRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::UserPermissionGrant;
const METHOD: &str = "create_user_permission_grant";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn create_user_permission_grant<T>(
        &self,
        user_permission_grant: T,
    ) -> Result<CreateUserPermissionRes, BoxedError>
    where
        T: Into<CreateUserPermissionReq> + fmt::Debug,
    {
        let user_permission_grant = user_permission_grant.into();

        let resp: Response<CreateUserPermissionRes> = self
            .post(
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
