use uuid::Uuid;
use oxidauth_http::{
    response::Response,
    server::api::v1::authorities::delete_authority::{DeleteAuthorityRes},
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Authority;
const METHOD: &str = "delete_authority";

impl Client {
    async fn delete_authority<T>(
        &self,
        authority_id: T,
    ) -> Result<DeleteAuthorityRes, BoxedError>
        where
            T: Into<Uuid>,
    {
        let authority_id = authority_id.into();

        let resp: Response<DeleteAuthorityRes> = self
            .delete(
                &format!("/authorities/{}", authority_id),
                None::<()>
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
