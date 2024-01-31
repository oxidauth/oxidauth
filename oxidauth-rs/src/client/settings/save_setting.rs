use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::settings::save_setting::{
    SaveSettingReq, SaveSettingRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Setting;
const METHOD: &str = "save_setting";

impl Client {
    pub async fn save_setting<T>(
        &self,
        params: T,
    ) -> Result<SaveSettingRes, BoxedError>
    where
        T: Into<SaveSettingReq>,
    {
        let params = params.into();

        let resp: Response<SaveSettingRes> = self
            .post("/settings", params)
            .await?;

        let setting_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(setting_res)
    }
}
