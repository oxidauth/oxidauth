use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::public_keys::list_all_public_keys::ListAllPublicKeysRes;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::PublicKey;
const METHOD: &str = "list_all_public_keys";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn list_all_public_keys<T>(
        &self,
    ) -> Result<ListAllPublicKeysRes, BoxedError> {
        let resp: Response<ListAllPublicKeysRes> = self
            .get("/public_keys", None::<()>)
            .await?;

        let public_key_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(public_key_res)
    }
}
