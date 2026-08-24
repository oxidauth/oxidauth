use async_trait::async_trait;
use oxidauth_http::{
    response::Response,
    server::api::v1::totp::validate::{ValidateTOTPReq, ValidateTOTPRes},
};

use oxidauth_kernel::error::BoxedError;

const RESOURCE: Resource = Resource::Totp;
const METHOD: &str = "validate";

use super::*;

#[async_trait]
pub trait ValidateTOTPTrait {
    async fn validate_totp<T>(&self, params: T) -> Result<ValidateTOTPRes, BoxedError>
    where
        T: Into<ValidateTOTPReq> + fmt::Debug + Send;
}

#[async_trait]
impl ValidateTOTPTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn validate_totp<T>(&self, params: T) -> Result<ValidateTOTPRes, BoxedError>
    where
        T: Into<ValidateTOTPReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<ValidateTOTPRes> = self
            .post("/totp/validate", params)
            .await?;

        let validate_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(validate_res)
    }
}
