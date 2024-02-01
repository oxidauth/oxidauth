use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::refresh_tokens::exchange::{
    ExchangeRefreshTokenReq, ExchangeRefreshTokenRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::RefreshToken;
const METHOD: &str = "exchange_refresh_token";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn exchange_refresh_token<T>(
        &self,
        params: T,
    ) -> Result<ExchangeRefreshTokenRes, BoxedError>
    where
        T: Into<ExchangeRefreshTokenReq> + fmt::Debug,
    {
        let params = params.into();

        let resp: Response<ExchangeRefreshTokenRes> = self
            .post("/refresh_tokens", params)
            .await?;

        let refresh_token_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(refresh_token_res)
    }
}
