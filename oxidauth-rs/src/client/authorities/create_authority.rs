use oxidauth_http::{
    response::Response,
    server::api::v1::authorities::create_authority::{CreateAuthorityReq, CreateAuthorityRes},
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Authority;
const METHOD: &str = "create_authority";

impl Client {
    async fn create_authority<T>(
        &self,
        authority: T,
    ) -> Result<CreateAuthorityRes, BoxedError>
        where
            T: Into<CreateAuthorityReq>,
    {
        let authority = authority.into();

        let resp: Response<CreateAuthorityRes> = self
            .post("/authorities", authority)
            .await?;

        let authority_res = handle_response(
            RESOURCE,
            METHOD,
            resp,
        )?;

        Ok(authority_res)
    }
}
