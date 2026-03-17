use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::settings::save_setting::{
    SaveSettingReq, SaveSettingRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Setting;
const METHOD: &str = "save_setting";

#[async_trait]
pub trait SaveSettingTrait {
    async fn save_setting<T>(
        &self,
        params: T,
    ) -> Result<SaveSettingRes, BoxedError>
    where
        T: Into<SaveSettingReq> + fmt::Debug + Send;
}

#[async_trait]
impl SaveSettingTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn save_setting<T>(
        &self,
        params: T,
    ) -> Result<SaveSettingRes, BoxedError>
    where
        T: Into<SaveSettingReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<SaveSettingRes> = self
            .post("/settings", params)
            .await?;

        let setting_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(setting_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl SaveSettingTrait for ClientMock {
    async fn save_setting<T>(
        &self,
        params: T,
    ) -> Result<SaveSettingRes, BoxedError>
    where
        T: Into<SaveSettingReq> + fmt::Debug + Send,
    {
        let Some(func) = self.save_setting_fn.clone() else {
            panic!("save_setting not defined for mock client");
        };

        return func(params.into());
    }
}
