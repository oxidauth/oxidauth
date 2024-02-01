use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::roles::create_user_role::CreateUserRoleRes;
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::UserRole;
const METHOD: &str = "create_user_role";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn create_user_role<T, R>(
        &self,
        user_id: T,
        role_id: R,
    ) -> Result<CreateUserRoleRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug,
        R: Into<Uuid> + fmt::Debug,
    {
        let user_id = user_id.into();
        let role_id = role_id.into();

        let resp: Response<CreateUserRoleRes> = self
            .post(
                &format!(
                    "/users/{}/roles/{}",
                    user_id, role_id
                ),
                None::<()>,
            )
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}
