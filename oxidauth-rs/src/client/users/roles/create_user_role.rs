use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::roles::create_user_role::CreateUserRoleRes;
use oxidauth_kernel::error::BoxedError;
use serde::Serialize;
use uuid::Uuid;

use super::*;

#[derive(Debug, Serialize)]
pub struct CreateUserRole {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

const RESOURCE: Resource = Resource::UserRole;
const METHOD: &str = "create_user_role";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn create_user_role<T>(
        &self,
        params: T,
    ) -> Result<CreateUserRoleRes, BoxedError>
    where
        T: Into<CreateUserRole> + fmt::Debug,
    {
        let CreateUserRole { user_id, role_id } = params.into();

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
