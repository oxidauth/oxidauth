use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::can::CanReq;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Permission;
const METHOD: &str = "can";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn can<T>(&self, params: T) -> Result<bool, BoxedError>
    where
        T: Into<CanReq> + fmt::Debug,
    {
        let params = params.into();

        let resp: Response<bool> = self
            .get(
                &format!("/can/{}", params.permission),
                None::<CanReq>,
            )
            .await?;

        let can_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(can_res)
    }
}
