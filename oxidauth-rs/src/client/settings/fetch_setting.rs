use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::settings::fetch_setting::{
    FetchSettingReq,
    FetchSettingRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Setting;
const METHOD: &str = "fetch_setting";

#[async_trait]
pub trait FetchSettingTrait {
    async fn fetch_setting<T>(
        &self,
        params: T,
    ) -> Result<FetchSettingRes, BoxedError>
    where
        T: Into<FetchSettingReq> + fmt::Debug + Send;
}

#[async_trait]
impl FetchSettingTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn fetch_setting<T>(
        &self,
        params: T,
    ) -> Result<FetchSettingRes, BoxedError>
    where
        T: Into<FetchSettingReq> + fmt::Debug + Send,
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

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl FetchSettingTrait for ClientMock {
    async fn fetch_setting<T>(
        &self,
        params: T,
    ) -> Result<FetchSettingRes, BoxedError>
    where
        T: Into<FetchSettingReq> + fmt::Debug + Send,
    {
        let Some(func) = self.fetch_setting_fn.clone() else {
            panic!("fetch_setting not defined for mock client");
        };

        return func(params.into());
    }
}
