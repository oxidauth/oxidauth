use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    settings::{Setting, fetch_setting::FetchSettingParams},
};

#[async_trait]
pub trait SelectSettingByKey: Send + Sync + 'static {
    async fn select_setting_by_key(
        &self,
        params: &FetchSettingParams,
    ) -> Result<Option<Setting>, BoxedError>;
}
