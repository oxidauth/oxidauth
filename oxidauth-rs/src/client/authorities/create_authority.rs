use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::authorities::create_authority::{
    CreateAuthority, CreateAuthorityReq, CreateAuthorityRes,
};
pub use oxidauth_kernel::authorities::TotpSettings;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Authority;
const METHOD: &str = "create_authority";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn create_authority<T>(
        &self,
        authority: T,
    ) -> Result<CreateAuthorityRes, BoxedError>
    where
        T: Into<CreateAuthorityReq> + fmt::Debug,
    {
        let authority = authority.into();

        let resp: Response<CreateAuthorityRes> = self
            .post("/authorities", authority)
            .await?;

        let authority_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(authority_res)
    }
}
