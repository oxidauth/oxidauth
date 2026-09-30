use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    settings::{Setting, save_setting::SaveSettingParams},
};

#[async_trait]
pub trait SaveSettingQuery: Send + Sync + 'static {
    async fn save_setting(&self, params: &SaveSettingParams) -> Result<Setting, BoxedError>;
}
