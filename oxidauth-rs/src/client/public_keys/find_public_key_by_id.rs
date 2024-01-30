use uuid::Uuid;

use oxidauth_http::{
    response::Response,
    server::api::v1::public_keys::find_public_key_by_id::FindPublicKeyByIdRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::PublicKey;
const METHOD: &str = "find_public_key_by_id";

impl Client {
    pub async fn find_public_key_by_id<T>(
        &self,
        public_key_id: T,
    ) -> Result<FindPublicKeyByIdRes, BoxedError>
    where
        T: Into<Uuid>,
    {
        let public_key_id = public_key_id.into();

        let resp: Response<FindPublicKeyByIdRes> = self
            .get(
                &format!("/public_keys/{}", public_key_id),
                None::<Uuid>,
            )
            .await?;

        let public_key_res = handle_response(
            RESOURCE,
            METHOD,
            resp,
        )?;

        Ok(public_key_res)
    }
}
