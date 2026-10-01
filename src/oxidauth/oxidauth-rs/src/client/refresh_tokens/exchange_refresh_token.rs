use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::refresh_tokens::exchange::{
    ExchangeRefreshTokenReq,
    ExchangeRefreshTokenRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::RefreshToken;
const METHOD: &str = "exchange_refresh_token";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ExchangeRefreshTokenTrait {
    async fn exchange_refresh_token<T>(
        &self,
        params: T,
    ) -> Result<ExchangeRefreshTokenRes, BoxedError>
    where
        T: Into<ExchangeRefreshTokenReq> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl ExchangeRefreshTokenTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn exchange_refresh_token<T>(
        &self,
        params: T,
    ) -> Result<ExchangeRefreshTokenRes, BoxedError>
    where
        T: Into<ExchangeRefreshTokenReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<ExchangeRefreshTokenRes> = self
            .post("/refresh_tokens", params)
            .await?;

        let refresh_token_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(refresh_token_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl ExchangeRefreshTokenTrait for ClientMock {
    async fn exchange_refresh_token<T>(
        &self,
        params: T,
    ) -> Result<ExchangeRefreshTokenRes, BoxedError>
    where
        T: Into<ExchangeRefreshTokenReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .exchange_refresh_token_fn
            .clone()
        else {
            panic!("exchange_refresh_token not defined for mock client");
        };

        return func(params.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, uid};

    #[tokio::test]
    async fn exchange_refresh_token_route_contract() {
        let refresh_token = Uuid::new_v4();

        contract(
            "POST",
            "/api/v1/refresh_tokens",
            ("refresh_token", "exchange_refresh_token"),
            json!({
                "jwt": "header.payload.signature",
                "refresh_token": uid(),
                "user_id": uid(),
            }),
            move |client| {
                async move {
                    client
                        .exchange_refresh_token(ExchangeRefreshTokenReq { refresh_token })
                        .await
                }
            },
        )
        .await;
    }
}
