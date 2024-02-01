use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::authorities::find_authority_by_id::FindAuthorityByIdRes;
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::Authority;
const METHOD: &str = "find_authority_by_id";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn find_authority_by_id<T>(
        &self,
        authority_id: T,
    ) -> Result<FindAuthorityByIdRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug,
    {
        let authority_id = authority_id.into();

        let resp: Response<FindAuthorityByIdRes> = self
            .get(
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
