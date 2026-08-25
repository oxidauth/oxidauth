use crate::response::Response;
use async_trait::async_trait;
use log::info;

use super::*;

const RESOURCE: Resource = Resource::RefreshToken;
const METHOD: &str = "exchange_refresh_token";

#[async_trait(?Send)]
#[async_trait]
pub trait ExchangeRefreshTokenTrait {
    async fn exchange_refresh_token<T>(&self, params: T) -> Result<ExchangeRefreshTokenRes, String>
    where
        T: Into<ExchangeRefreshTokenReq> + fmt::Debug;
}

#[async_trait(?Send)]
#[async_trait]
impl ExchangeRefreshTokenTrait for Client {
    async fn exchange_refresh_token<T>(&self, params: T) -> Result<ExchangeRefreshTokenRes, String>
    where
        T: Into<ExchangeRefreshTokenReq> + fmt::Debug,
    {
        info!("IN CLIENT/REFRESH_TOKENS/EXCHANGE_REFRESH_TOKEN - START");

        let params = params.into();

        let result = self
            .post("/refresh_tokens", params)
            .await;

        let Ok(resp) = result else {
            return Err("exchange_refresh_token call failed".to_string());
        };

        info!("IN CLIENT/REFRESH_TOKENS/EXCHANGE_REFRESH_TOKEN - HANDLE RESPONSE");
        let refresh_token_res = handle_response(RESOURCE, METHOD, resp);

        let Ok(refresh_token_res) = refresh_token_res else {
            return Err("exchange_refresh_token call failed".to_string());
        };

        Ok(refresh_token_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl ExchangeRefreshTokenTrait for ClientMock {
    async fn exchange_refresh_token<T>(&self, params: T) -> Result<ExchangeRefreshTokenRes, String>
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
