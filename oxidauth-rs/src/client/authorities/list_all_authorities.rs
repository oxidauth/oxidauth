use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::authorities::list_all_authorities::{
    ListAllAuthoritiesReq, ListAllAuthoritiesRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Authority;
const METHOD: &str = "find_authority_by_strategy";

impl Client {
    pub async fn list_all_authorities<T>(
        &self,
        params: T,
    ) -> Result<ListAllAuthoritiesRes, BoxedError>
    where
        T: Into<ListAllAuthoritiesReq>,
    {
        let params = params.into();

        let resp: Response<ListAllAuthoritiesRes> = self
            .get("/authorities", params)
            .await?;

        let authority_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(authority_res)
    }
}
