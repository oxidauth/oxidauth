use async_trait::async_trait;
use oxidauth_http::response::Response;
use oxidauth_kernel::error::BoxedError;

use tracing::info;

use super::*;

const RESOURCE: Resource = Resource::RefreshToken;
const METHOD: &str = "exchange_refresh_token";

#[async_trait]
pub trait ExchangeRefreshTokenTrait {
    async fn exchange_refresh_token<T>(
        &self,
        params: T,
    ) -> Result<ExchangeRefreshTokenRes, BoxedError>
    where
        T: Into<ExchangeRefreshTokenReq> + fmt::Debug + Send;
}

#[async_trait]
impl ExchangeRefreshTokenTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn exchange_refresh_token<T>(
        &self,
        params: T,
    ) -> Result<ExchangeRefreshTokenRes, BoxedError>
    where
        T: Into<ExchangeRefreshTokenReq> + fmt::Debug + Send,
    {
        info!("IN CLIENT/REFRESH_TOKENS/EXCHANGE_REFRESH_TOKEN - START");

        let params = params.into();

        let resp: Response<ExchangeRefreshTokenRes> = self
            .post("/refresh_tokens", params)
            .await?;

        info!("IN CLIENT/REFRESH_TOKENS/EXCHANGE_REFRESH_TOKEN - HANDLE RESPONSE");
        let refresh_token_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(refresh_token_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
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
