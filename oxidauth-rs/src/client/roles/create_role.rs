use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::roles::create_role::{
    CreateRole, CreateRoleReq, CreateRoleRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Role;
const METHOD: &str = "create_role";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn create_role<T>(
        &self,
        role: T,
    ) -> Result<CreateRoleRes, BoxedError>
    where
        T: Into<CreateRoleReq> + fmt::Debug,
    {
        let role = role.into();

        let resp: Response<CreateRoleRes> = self
            .post("/roles", role)
            .await?;

        let role_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_res)
    }
}
