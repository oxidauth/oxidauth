use uuid::Uuid;

use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::roles::delete_role::DeleteRoleRes;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Role;
const METHOD: &str = "delete_role";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn delete_role<T>(
        &self,
        role_id: T,
    ) -> Result<DeleteRoleRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug,
    {
        let role_id = role_id.into();

        let resp: Response<DeleteRoleRes> = self
            .delete(
                &format!("/roles/{}", role_id),
                None::<Uuid>,
            )
            .await?;

        let role_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_res)
    }
}
