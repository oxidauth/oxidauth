use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::roles::find_role_by_name::FindRoleByNameRes;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Role;
const METHOD: &str = "find_role_by_name";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn find_role_by_name<T>(
        &self,
        role: T,
    ) -> Result<FindRoleByNameRes, BoxedError>
    where
        T: Into<String> + fmt::Debug,
    {
        let role = role.into();

        let resp: Response<FindRoleByNameRes> = self
            .get(
                &format!("/roles/{}", role),
                None::<()>,
            )
            .await?;

        let role_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_res)
    }
}
