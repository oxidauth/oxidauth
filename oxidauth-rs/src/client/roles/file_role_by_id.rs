use uuid::Uuid;

use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::roles::find_role_by_id::FindRoleByIdRes;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Role;
const METHOD: &str = "find_role_by_id";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn find_role_by_id<T>(
        &self,
        role_id: T,
    ) -> Result<FindRoleByIdRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug,
    {
        let role_id = role_id.into();

        let resp: Response<FindRoleByIdRes> = self
            .get(
                &format!("/roles/{}", role_id),
                None::<Uuid>,
            )
            .await?;

        let role_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_res)
    }
}
