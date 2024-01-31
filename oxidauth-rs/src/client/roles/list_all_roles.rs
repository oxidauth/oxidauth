use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::roles::list_all_roles::{
    ListAllRolesReq, ListAllRolesRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Role;
const METHOD: &str = "list_all_roles";

impl Client {
    async fn list_all_roles<T>(
        &self,
        params: T,
    ) -> Result<ListAllRolesRes, BoxedError>
    where
        T: Into<ListAllRolesReq>,
    {
        let _params = params.into();

        let resp: Response<ListAllRolesRes> = self
            .get(
                "/roles",
                None::<ListAllRolesReq>,
            )
            .await?;

        let role_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_res)
    }
}
