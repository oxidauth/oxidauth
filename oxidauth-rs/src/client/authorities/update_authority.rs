use uuid::Uuid;
use oxidauth_http::{
    response::Response,
    server::api::v1::authorities::update_authority::{UpdateAuthorityReq, UpdateAuthorityRes},
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Authority;
const METHOD: &str = "update_authority";

impl Client {
    async fn update_authority<T, U>(
        &self,
        authority_id: U,
        params: T,
    ) -> Result<UpdateAuthorityRes, BoxedError>
        where
            T: Into<UpdateAuthorityReq>,
            U: Into<Uuid>,
    {
        let authority_id = authority_id.into();
        let params = params.into();

        let resp: Response<UpdateAuthorityRes> = self
            .post(
                &format!("/authorities/{}", authority_id),
                params,
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