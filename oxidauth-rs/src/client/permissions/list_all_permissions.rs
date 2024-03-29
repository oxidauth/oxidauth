use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::permissions::list_all_permissions::{
    ListAllPermissionsReq, ListAllPermissionsRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Permission;
const METHOD: &str = "list_all_permissions";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn list_all_permissions<T>(
        &self,
        params: T,
    ) -> Result<ListAllPermissionsRes, BoxedError>
    where
        T: Into<ListAllPermissionsReq> + fmt::Debug,
    {
        let params = params.into();

        let resp: Response<ListAllPermissionsRes> = self
            .get("/permissions", params)
            .await?;

        let permission_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(permission_res)
    }
}
