use oxidauth_http::{
    response::Response,
    server::api::v1::settings::fetch_setting::{FetchSettingReq, FetchSettingRes},
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Setting;
const METHOD: &str = "fetch_setting";

impl Client {
    pub async fn fetch_setting<T>(
        &self,
        params: T,
    ) -> Result<FetchSettingRes, BoxedError>
    where
        T: Into<FetchSettingReq>,
    {
        let params = params.into();

        let resp: Response<FetchSettingRes> = self
            .get(
                &format!("/settings/{}", params.key),
                None::<FetchSettingReq>,
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
