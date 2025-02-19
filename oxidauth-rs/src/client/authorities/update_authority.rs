use uuid::Uuid;

use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::authorities::update_authority::{
    UpdateAuthority, UpdateAuthorityPathReq, UpdateAuthorityReq,
    UpdateAuthorityRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Authority;
const METHOD: &str = "update_authority";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn update_authority<T, U>(
        &self,
        authority_id: U,
        params: T,
    ) -> Result<UpdateAuthorityRes, BoxedError>
    where
        U: Into<Uuid> + fmt::Debug,
        T: Into<UpdateAuthorityReq> + fmt::Debug,
    {
        let authority_id = authority_id.into();
        let params = params.into();

        let resp: Response<UpdateAuthorityRes> = self
            .put(
                &format!(
                    "/authorities/{}",
                    authority_id
                ),
                params,
            )
            .await?;

        let authority_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(authority_res)
    }
}
