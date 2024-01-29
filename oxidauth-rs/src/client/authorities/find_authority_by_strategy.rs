use oxidauth_http::{
    response::Response,
    server::api::v1::authorities::find_authority_by_strategy::FindAuthorityByStrategyRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Authority;
const METHOD: &str = "find_authority_by_strategy";

impl Client {
    async fn find_authority_by_strategy<T>(
        &self,
        authority_strategy: T,
    ) -> Result<FindAuthorityByStrategyRes, BoxedError>
        where
            T: Into<String>,
    {
        let authority_strategy = authority_strategy.into();

        let resp: Response<FindAuthorityByStrategyRes> = self
            .get(
                &format!("/authorities/by_strategy/{}", authority_strategy),
                None::<()>,
            )
            .await?;

        let authority_res = handle_response(
            RESOURCE,
            METHOD,
            resp,
        )?;

        Ok(authority_res)
    }
}
