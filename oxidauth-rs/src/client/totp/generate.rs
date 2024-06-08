use oxidauth_http::{
    response::Response,
    server::api::v1::totp::generate::{TOTPGenerateReq, TOTPGenerateRes},
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Totp;
const METHOD: &str = "generate_code";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn generate_totp_code<T>(
        &self,
        params: T,
    ) -> Result<TOTPGenerateRes, BoxedError>
    where
        T: Into<TOTPGenerateReq> + fmt::Debug,
    {
        let params = params.into();

        let resp: Response<TOTPGenerateRes> = self
            .post("/totp/generate", params)
            .await?;

        let totp_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(totp_res)
    }
}
