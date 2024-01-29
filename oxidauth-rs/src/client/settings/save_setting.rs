use oxidauth_http::{
    response::Response,
    server::api::v1::settings::save_setting::{SaveSettingReq, SaveSettingRes},
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Setting;
const METHOD: &str = "save_setting";

impl Client {
    async fn save_setting<T>(
        &self,
        params: T,
    ) -> Result<SaveSettingRes, BoxedError>
    where
        T: Into<SaveSettingReq>,
    {
        let params = params.into();

        let resp: Response<SaveSettingRes> = self
            .post(
                "/settings",
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
