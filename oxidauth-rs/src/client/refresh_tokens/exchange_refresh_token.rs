use oxidauth_http::{
    response::Response,
    server::api::v1::refresh_tokens::exchange::{ExchangeRefreshTokenReq, ExchangeRefreshTokenRes},
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::RefreshToken;
const METHOD: &str = "exchange_refresh_token";

impl Client {
    pub async fn exchange_refresh_token<T>(
        &self,
        params: T,
    ) -> Result<ExchangeRefreshTokenRes, BoxedError>
    where
        T: Into<ExchangeRefreshTokenReq>,
    {
        let params = params.into();

        let resp: Response<ExchangeRefreshTokenRes> = self
            .post(
                "/refresh_tokens",
                params,
            )
            .await?;

        let role_res = handle_response(
            RESOURCE,
            METHOD,
            resp,
        )?;

        Ok(role_res)
    }
}
