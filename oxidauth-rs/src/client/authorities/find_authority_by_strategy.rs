use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::authorities::find_authority_by_strategy::{
    FindAuthorityByStrategyReq, FindAuthorityByStrategyRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Authority;
const METHOD: &str = "find_authority_by_strategy";

impl Client {
    pub async fn find_authority_by_strategy<T>(
        &self,
        authority: T,
    ) -> Result<FindAuthorityByStrategyRes, BoxedError>
    where
        T: Into<FindAuthorityByStrategyReq>,
    {
        let authority = authority.into();

        let resp: Response<FindAuthorityByStrategyRes> = self
            .get("/authorities", authority)
            .await?;

        let authority_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(authority_res)
    }
}
