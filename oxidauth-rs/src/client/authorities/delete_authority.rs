use uuid::Uuid;

use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::authorities::delete_authority::DeleteAuthorityRes;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Authority;
const METHOD: &str = "delete_authority";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn delete_authority<T>(
        &self,
        authority_id: T,
    ) -> Result<DeleteAuthorityRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug,
    {
        let authority_id = authority_id.into();

        let resp: Response<DeleteAuthorityRes> = self
            .delete(
                &format!(
                    "/authorities/{}",
                    authority_id
                ),
                None::<()>,
            )
            .await?;

        let authority_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(authority_res)
    }
}
