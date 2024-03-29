use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::public_keys::delete_public_key::DeletePublicKeyRes;
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::PublicKey;
const METHOD: &str = "delete_public_key";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn delete_public_key<T>(
        &self,
        public_key_id: T,
    ) -> Result<DeletePublicKeyRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug,
    {
        let public_key_id = public_key_id.into();

        let resp: Response<DeletePublicKeyRes> = self
            .delete(
                &format!(
                    "/public_keys/{}",
                    public_key_id
                ),
                None::<Uuid>,
            )
            .await?;

        let public_key_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(public_key_res)
    }
}
