use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::authorities::delete_authority::DeleteAuthorityRes;
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::Authority;
const METHOD: &str = "delete_authority";

#[async_trait]
pub trait DeleteAuthorityTrait {
    async fn delete_authority<T>(&self, authority_id: T) -> Result<DeleteAuthorityRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send;
}

#[async_trait]
impl DeleteAuthorityTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn delete_authority<T>(&self, authority_id: T) -> Result<DeleteAuthorityRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
    {
        let authority_id = authority_id.into();

        let resp: Response<DeleteAuthorityRes> = self
            .delete(&format!("/authorities/{}", authority_id), None::<()>)
            .await?;

        let authority_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(authority_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl DeleteAuthorityTrait for ClientMock {
    async fn delete_authority<T>(&self, authority_id: T) -> Result<DeleteAuthorityRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
    {
        let Some(func) = self
            .delete_authority_fn
            .clone()
        else {
            panic!("delete_authority not defined for mock client");
        };

        return func(authority_id.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{authority, contract};

    #[tokio::test]
    async fn delete_authority_route_contract() {
        let authority_id = Uuid::new_v4();

        contract(
            "DELETE",
            &format!("/api/v1/authorities/{authority_id}"),
            ("authority", "delete_authority"),
            json!({ "authority": authority() }),
            move |client| {
                async move {
                    client
                        .delete_authority(authority_id)
                        .await
                }
            },
        )
        .await;
    }
}
