use uuid::Uuid;
pub use oxidauth_http::{
    response::Response,
    server::api::v1::authorities::find_authority_by_id::FindAuthorityByIdRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Authority;
const METHOD: &str = "find_authority_by_id";

impl Client {
    pub async fn find_authority_by_id<T>(
        &self,
        authority_id: T,
    ) -> Result<FindAuthorityByIdRes, BoxedError>
        where
            T: Into<Uuid>,
    {
        let authority_id = authority_id.into();

        let resp: Response<FindAuthorityByIdRes> = self
            .get(
                &format!("/authorities/{}", authority_id),
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
