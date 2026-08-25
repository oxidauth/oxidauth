const RESOURCE: Resource = Resource::Totp;
const METHOD: &str = "validate";

use super::*;

#[async_trait(?Send)]
#[async_trait]
pub trait ValidateTOTPTrait {
    async fn validate_totp<T>(&self, params: T) -> Result<ValidateTOTPRes, String>
    where
        T: Into<ValidateTOTPReq> + fmt::Debug;
}

#[async_trait(?Send)]
#[async_trait]
impl ValidateTOTPTrait for Client {
    async fn validate_totp<T>(&self, params: T) -> Result<ValidateTOTPRes, String>
    where
        T: Into<ValidateTOTPReq> + fmt::Debug,
    {
        let params = params.into();

        let resp = self
            .post("/totp/validate", params)
            .await;

        let Ok(result) = resp else {
            return Err("validate_totp call failed".to_string());
        };

        let validate_res = handle_response(RESOURCE, METHOD, result);

        let Ok(validate_res) = validate_res else {
            return Err("validate_totp call failed".to_string());
        };

        Ok(validate_res)
    }
}
