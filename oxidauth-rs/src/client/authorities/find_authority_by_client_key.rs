pub use oxidauth_http::server::api::v1::authorities::find_authority_by_client_key::FindAuthorityByClientKeyRes;
pub use oxidauth_kernel::authorities::find_authority_by_client_key::FindAuthorityByClientKey;
use oxidauth_kernel::error::BoxedError;
use oxidauth_http::response::Response;

use super::*;

const RESOURCE: Resource = Resource::Authority;
const METHOD: &str = "find_authority_by_client_key";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn find_authority_by_client_key<T>(
        &self,
        authority_client_key: T,
    ) -> Result<FindAuthorityByClientKeyRes, BoxedError>
    where
        T: Into<String> + fmt::Debug,
    {
        let authority_client_key = authority_client_key.into();

        let resp: Response<FindAuthorityByClientKeyRes> = self
            .get(
                &format!(
                    "/authorities/by_client_key/{}",
                    authority_client_key
                ),
                None::<()>,
            )
            .await?;

        let authority_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(authority_res)
    }
}
