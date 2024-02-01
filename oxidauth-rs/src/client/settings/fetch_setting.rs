use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::settings::fetch_setting::{
    FetchSettingReq, FetchSettingRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Setting;
const METHOD: &str = "fetch_setting";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn fetch_setting<T>(
        &self,
        params: T,
    ) -> Result<FetchSettingRes, BoxedError>
    where
        T: Into<FetchSettingReq> + fmt::Debug,
    {
        let params = params.into();

        let resp: Response<FetchSettingRes> = self
            .get(
                &format!("/settings/{}", params.key),
                None::<FetchSettingReq>,
            )
            .await?;

        let setting_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(setting_res)
    }
}
