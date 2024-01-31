use uuid::Uuid;

use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::roles::list_user_roles_by_user_id::ListUserRoleGrantsByUserIdRes;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::UserRole;
const METHOD: &str = "list_user_roles_by_user_id";

impl Client {
    pub async fn list_user_roles_by_user_id<T>(
        &self,
        user_id: T,
    ) -> Result<ListUserRoleGrantsByUserIdRes, BoxedError>
    where
        T: Into<Uuid>,
    {
        let user_id = user_id.into();

        let resp: Response<ListUserRoleGrantsByUserIdRes> = self
            .get(
                &format!("/users/{}/roles", user_id),
                None::<()>,
            )
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}
