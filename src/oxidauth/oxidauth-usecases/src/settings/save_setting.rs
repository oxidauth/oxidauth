use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    service::Service,
    settings::{save_setting::SaveSettingParams, Setting},
};
use oxidauth_repository::settings::upsert_setting::SaveSettingQuery;

pub struct SaveSettingUseCase<T>
where
    T: SaveSettingQuery,
{
    save_setting: T,
}

impl<T> SaveSettingUseCase<T>
where
    T: SaveSettingQuery,
{
    pub fn new(save_setting: T) -> Self {
        Self { save_setting }
    }
}

#[async_trait]
impl<'a, T> Service<&'a SaveSettingParams> for SaveSettingUseCase<T>
where
    T: SaveSettingQuery,
{
    type Response = Setting;
    type Error = BoxedError;

    #[tracing::instrument(name = "save_setting_usecase", skip(self))]
    async fn call(
        &self,
        params: &'a SaveSettingParams,
    ) -> Result<Self::Response, Self::Error> {
        self.save_setting
            .call(params)
            .await
    }
}
