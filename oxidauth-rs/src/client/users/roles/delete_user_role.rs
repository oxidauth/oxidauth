use uuid::Uuid;

use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::roles::delete_user_role::DeleteUserRoleRes;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::UserRole;
const METHOD: &str = "delete_user_role";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn delete_user_role<T, R>(
        &self,
        user_id: T,
        role_id: R,
    ) -> Result<DeleteUserRoleRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug,
        R: Into<Uuid> + fmt::Debug,
    {
        let user_id = user_id.into();
        let role_id = role_id.into();

        let resp: Response<DeleteUserRoleRes> = self
            .delete(
                &format!(
                    "/users/{}/roles/{}",
                    user_id, role_id
                ),
                None::<()>,
            )
            .await?;

        let user_role_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_role_res)
    }
}
