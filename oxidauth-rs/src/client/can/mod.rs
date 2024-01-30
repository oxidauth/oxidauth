use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::can::{CanReq, CanRes};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Permission;
const METHOD: &str = "can";

impl Client {
    pub async fn can<T>(&self, params: T) -> Result<CanRes, BoxedError>
    where
        T: Into<CanReq>,
    {
        let params = params.into();

        let resp: Response<CanRes> = self
            .get(
                &format!("/can/{}", params.permission),
                None::<CanReq>,
            )
            .await?;

        let can_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(can_res)
    }
}
