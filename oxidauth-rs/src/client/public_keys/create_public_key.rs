use oxidauth_http::{
    response::Response,
    server::api::v1::public_keys::create_public_key::CreatePublicKeyRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::PublicKey;
const METHOD: &str = "create_public_key";

impl Client {
    async fn create_public_key<T>(
        &self,
    ) -> Result<CreatePublicKeyRes, BoxedError> {
        let resp: Response<CreatePublicKeyRes> = self
            .post("/public_keys", None::<()>)
            .await?;

        let public_key_res = handle_response(
            RESOURCE,
            METHOD,
            resp,
        )?;

        Ok(public_key_res)
    }
}
