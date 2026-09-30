use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::public_keys::delete_public_key::DeletePublicKeyRes;
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::PublicKey;
const METHOD: &str = "delete_public_key";

#[async_trait]
pub trait DeletePublicKeyTrait {
    async fn delete_public_key<T>(
        &self,
        public_key_id: T,
    ) -> Result<DeletePublicKeyRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send;
}

#[async_trait]
impl DeletePublicKeyTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn delete_public_key<T>(&self, public_key_id: T) -> Result<DeletePublicKeyRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
    {
        let public_key_id = public_key_id.into();

        let resp: Response<DeletePublicKeyRes> = self
            .delete(&format!("/public_keys/{}", public_key_id), None::<Uuid>)
            .await?;

        let public_key_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(public_key_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl DeletePublicKeyTrait for ClientMock {
    async fn delete_public_key<T>(&self, public_key_id: T) -> Result<DeletePublicKeyRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
    {
        let Some(func) = self
            .delete_public_key_fn
            .clone()
        else {
            panic!("delete_public_key not defined for mock client");
        };

        return func(public_key_id.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, public_key};

    #[tokio::test]
    async fn delete_public_key_route_contract() {
        let public_key_id = Uuid::new_v4();

        contract(
            "DELETE",
            &format!("/api/v1/public_keys/{public_key_id}"),
            ("public_key", "delete_public_key"),
            json!({ "public_key": public_key() }),
            move |client| {
                async move {
                    client
                        .delete_public_key(public_key_id)
                        .await
                }
            },
        )
        .await;
    }
}
