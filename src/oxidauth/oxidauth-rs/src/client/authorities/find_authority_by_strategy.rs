use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::authorities::find_authority_by_strategy::FindAuthorityByStrategyRes;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Authority;
const METHOD: &str = "find_authority_by_strategy";

#[async_trait]
pub trait FindAuthorityByStrategyTrait {
    async fn find_authority_by_strategy<T>(
        &self,
        authority_strategy: T,
    ) -> Result<FindAuthorityByStrategyRes, BoxedError>
    where
        T: Into<String> + fmt::Debug + Send;
}

#[async_trait]
impl FindAuthorityByStrategyTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn find_authority_by_strategy<T>(
        &self,
        authority_strategy: T,
    ) -> Result<FindAuthorityByStrategyRes, BoxedError>
    where
        T: Into<String> + fmt::Debug + Send,
    {
        let authority_strategy = authority_strategy.into();

        let resp: Response<FindAuthorityByStrategyRes> = self
            .get(
                &format!("/authorities/by_strategy/{}", authority_strategy),
                None::<()>,
            )
            .await?;

        let authority_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(authority_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl FindAuthorityByStrategyTrait for ClientMock {
    async fn find_authority_by_strategy<T>(
        &self,
        authority_strategy: T,
    ) -> Result<FindAuthorityByStrategyRes, BoxedError>
    where
        T: Into<String> + fmt::Debug + Send,
    {
        let Some(func) = self
            .find_authority_by_strategy_fn
            .clone()
        else {
            panic!("find_authority_by_strategy not defined for mock client");
        };

        return func(authority_strategy.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::client::users::contract::{authority, contract};

    #[tokio::test]
    async fn find_authority_by_strategy_route_contract() {
        contract(
            "GET",
            "/api/v1/authorities/by_strategy/username_password",
            ("authority", "find_authority_by_strategy"),
            json!({ "authority": authority() }),
            |client| {
                async move {
                    client
                        .find_authority_by_strategy("username_password".to_string())
                        .await
                }
            },
        )
        .await;
    }
}
