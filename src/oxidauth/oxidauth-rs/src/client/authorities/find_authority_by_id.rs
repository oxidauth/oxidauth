use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::authorities::find_authority_by_id::FindAuthorityByIdRes;
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::Authority;
const METHOD: &str = "find_authority_by_id";

#[async_trait]
pub trait FindAuthorityByIdTrait {
    async fn find_authority_by_id<T>(
        &self,
        authority_id: T,
    ) -> Result<FindAuthorityByIdRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send;
}

#[async_trait]
impl FindAuthorityByIdTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn find_authority_by_id<T>(
        &self,
        authority_id: T,
    ) -> Result<FindAuthorityByIdRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
    {
        let authority_id = authority_id.into();

        let resp: Response<FindAuthorityByIdRes> = self
            .get(&format!("/authorities/{}", authority_id), None::<()>)
            .await?;

        let authority_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(authority_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl FindAuthorityByIdTrait for ClientMock {
    async fn find_authority_by_id<T>(
        &self,
        authority_id: T,
    ) -> Result<FindAuthorityByIdRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
    {
        let Some(func) = self
            .find_authority_by_id_fn
            .clone()
        else {
            panic!("find_authority_by_id not defined for mock client");
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
    async fn find_authority_by_id_route_contract() {
        let authority_id = Uuid::new_v4();

        contract(
            "GET",
            &format!("/api/v1/authorities/{authority_id}"),
            ("authority", "find_authority_by_id"),
            json!({ "authority": authority() }),
            move |client| {
                async move {
                    client
                        .find_authority_by_id(authority_id)
                        .await
                }
            },
        )
        .await;
    }
}
